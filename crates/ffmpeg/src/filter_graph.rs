use std::ffi::CString;
use std::ptr::NonNull;

use ffmpeg_sys_next::*;

use crate::error::{FfmpegError, AVERROR_EAGAIN};
use crate::frame::Frame;
use crate::smart_object::SmartPtr;

pub struct FilterGraph(SmartPtr<AVFilterGraph>);

/// Safety: `FilterGraph` is safe to send between threads.
unsafe impl Send for FilterGraph {}

impl FilterGraph {
    pub fn new() -> Result<Self, FfmpegError> {
        // Safety: the pointer returned from avfilter_graph_alloc is valid
        unsafe { Self::wrap(avfilter_graph_alloc()) }
    }

    /// Safety: `ptr` must be a valid pointer to an `AVFilterGraph`.
    unsafe fn wrap(ptr: *mut AVFilterGraph) -> Result<Self, FfmpegError> {
        Ok(Self(
            SmartPtr::wrap_non_null(ptr, |ptr| unsafe { avfilter_graph_free(ptr) }).ok_or(FfmpegError::Alloc)?,
        ))
    }

    pub fn as_ptr(&self) -> *const AVFilterGraph {
        self.0.as_ptr()
    }

    pub fn as_mut_ptr(&mut self) -> *mut AVFilterGraph {
        self.0.as_mut_ptr()
    }

    pub fn add(&mut self, filter: Filter, name: &str, args: &str) -> Result<FilterContext<'_>, FfmpegError> {
        let name = CString::new(name).expect("failed to convert name to CString");
        let args = CString::new(args).expect("failed to convert args to CString");

        let mut filter_context = std::ptr::null_mut();

        // Safety: avfilter_graph_create_filter is safe to call, 'filter_context' is a
        // valid pointer
        let ret = unsafe {
            avfilter_graph_create_filter(
                &mut filter_context,
                filter.as_ptr(),
                name.as_ptr(),
                args.as_ptr(),
                std::ptr::null_mut(),
                self.as_mut_ptr(),
            )
        };

        if ret < 0 {
            Err(FfmpegError::Code(ret.into()))
        } else {
            // Safety: 'filter_context' is a valid pointer
            Ok(FilterContext(unsafe {
                NonNull::new(filter_context).ok_or(FfmpegError::Alloc)?.as_mut()
            }))
        }
    }

    pub fn get(&mut self, name: &str) -> Option<FilterContext<'_>> {
        let name = CString::new(name).unwrap();
        // Safety: avfilter_graph_get_filter is safe to call, and the returned pointer
        // is valid
        let mut ptr = NonNull::new(unsafe { avfilter_graph_get_filter(self.as_mut_ptr(), name.as_ptr()) })?;
        Some(FilterContext(unsafe { ptr.as_mut() }))
    }

    pub fn validate(&mut self) -> Result<(), FfmpegError> {
        // Safety: avfilter_graph_config is safe to call
        let ret = unsafe { avfilter_graph_config(self.as_mut_ptr(), std::ptr::null_mut()) };

        if ret < 0 {
            Err(FfmpegError::Code(ret.into()))
        } else {
            Ok(())
        }
    }

    pub fn dump(&mut self) -> Option<String> {
        unsafe {
            // Safety: avfilter_graph_dump is safe to call, and the returned pointer is
            // valid
            let c_str = SmartPtr::wrap_non_null(avfilter_graph_dump(self.as_mut_ptr(), std::ptr::null_mut()), |ptr| {
                av_free(*ptr as *mut libc::c_void);
                *ptr = std::ptr::null_mut();
            })?;

            // Safety: the lifetime of c_str does not exceed the lifetime of the the `CStr`
            // returned by `from_ptr`
            let c_str = std::ffi::CStr::from_ptr(c_str.as_ptr());
            Some(c_str.to_str().ok()?.to_owned())
        }
    }

    pub fn set_thread_count(&mut self, threads: i32) {
        self.0.as_deref_mut_except().nb_threads = threads;
    }

    pub fn input(&mut self, name: &str, pad: i32) -> Result<FilterGraphParser<'_>, FfmpegError> {
        FilterGraphParser::new(self).input(name, pad)
    }

    pub fn output(&mut self, name: &str, pad: i32) -> Result<FilterGraphParser<'_>, FfmpegError> {
        FilterGraphParser::new(self).output(name, pad)
    }
}

pub struct FilterGraphParser<'a> {
    graph: &'a mut FilterGraph,
    inputs: SmartPtr<AVFilterInOut>,
    outputs: SmartPtr<AVFilterInOut>,
}

/// Safety: `FilterGraphParser` is safe to send between threads.
unsafe impl Send for FilterGraphParser<'_> {}

impl<'a> FilterGraphParser<'a> {
    fn new(graph: &'a mut FilterGraph) -> Self {
        Self {
            graph,
            // Safety: 'avfilter_inout_free' is safe to call with a null pointer, and the pointer is valid
            inputs: unsafe { SmartPtr::wrap(std::ptr::null_mut(), |ptr| avfilter_inout_free(ptr)) },
            // Safety: 'avfilter_inout_free' is safe to call with a null pointer, and the pointer is valid
            outputs: unsafe { SmartPtr::wrap(std::ptr::null_mut(), |ptr| avfilter_inout_free(ptr)) },
        }
    }

    pub fn input(self, name: &str, pad: i32) -> Result<Self, FfmpegError> {
        self.inout_impl(name, pad, false)
    }

    pub fn output(self, name: &str, pad: i32) -> Result<Self, FfmpegError> {
        self.inout_impl(name, pad, true)
    }

    pub fn parse(mut self, spec: &str) -> Result<(), FfmpegError> {
        let spec = CString::new(spec).unwrap();

        // Safety: 'avfilter_graph_parse_ptr' is safe to call and all the pointers are
        // valid.
        unsafe {
            match avfilter_graph_parse_ptr(
                self.graph.as_mut_ptr(),
                spec.as_ptr(),
                self.inputs.as_mut(),
                self.outputs.as_mut(),
                std::ptr::null_mut(),
            ) {
                n if n >= 0 => Ok(()),
                e => Err(FfmpegError::Code(e.into())),
            }
        }
    }

    fn inout_impl(mut self, name: &str, pad: i32, output: bool) -> Result<Self, FfmpegError> {
        let context = self.graph.get(name).ok_or(FfmpegError::Arguments("unknown name"))?;

        // Safety: 'avfilter_inout_alloc' is safe to call, and the returned pointer is
        // valid
        let mut inout = unsafe { SmartPtr::wrap_non_null(avfilter_inout_alloc(), |ptr| avfilter_inout_free(ptr)) }
            .ok_or(FfmpegError::Alloc)?;

        let name = CString::new(name).unwrap();

        inout.as_deref_mut_except().name = name.into_raw();
        inout.as_deref_mut_except().filter_ctx = context.0;
        inout.as_deref_mut_except().pad_idx = pad;

        if output {
            inout.as_deref_mut_except().next = self.outputs.into_inner();
            self.outputs = inout;
        } else {
            inout.as_deref_mut_except().next = self.inputs.into_inner();
            self.inputs = inout;
        }

        Ok(self)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Filter(*const AVFilter);

impl Filter {
    pub fn get(name: &str) -> Option<Self> {
        let name = std::ffi::CString::new(name).ok()?;

        // Safety: avfilter_get_by_name is safe to call, and the returned pointer is
        // valid
        let filter = unsafe { avfilter_get_by_name(name.as_ptr()) };

        if filter.is_null() {
            None
        } else {
            Some(Self(filter))
        }
    }

    pub fn as_ptr(&self) -> *const AVFilter {
        self.0
    }

    /// # Safety
    /// `ptr` must be a valid pointer.
    pub unsafe fn wrap(ptr: *const AVFilter) -> Self {
        Self(ptr)
    }
}

/// Safety: `Filter` is safe to send between threads.
unsafe impl Send for Filter {}

pub struct FilterContext<'a>(&'a mut AVFilterContext);

/// Safety: `FilterContext` is safe to send between threads.
unsafe impl Send for FilterContext<'_> {}

impl<'a> FilterContext<'a> {
    pub fn source(self) -> FilterContextSource<'a> {
        FilterContextSource(self.0)
    }

    pub fn sink(self) -> FilterContextSink<'a> {
        FilterContextSink(self.0)
    }
}

pub struct FilterContextSource<'a>(&'a mut AVFilterContext);

/// Safety: `FilterContextSource` is safe to send between threads.
unsafe impl Send for FilterContextSource<'_> {}

impl FilterContextSource<'_> {
    pub fn send_frame(&mut self, frame: &Frame) -> Result<(), FfmpegError> {
        // Safety: `frame` is a valid pointer, and `self.0` is a valid pointer.
        unsafe {
            match av_buffersrc_write_frame(self.0, frame.as_ptr()) {
                0 => Ok(()),
                e => Err(FfmpegError::Code(e.into())),
            }
        }
    }

    pub fn send_eof(&mut self, pts: Option<i64>) -> Result<(), FfmpegError> {
        // Safety: `self.0` is a valid pointer.
        unsafe {
            match if let Some(pts) = pts {
                av_buffersrc_close(self.0, pts, 0)
            } else {
                av_buffersrc_write_frame(self.0, std::ptr::null())
            } {
                0 => Ok(()),
                e => Err(FfmpegError::Code(e.into())),
            }
        }
    }
}

pub struct FilterContextSink<'a>(&'a mut AVFilterContext);

/// Safety: `FilterContextSink` is safe to send between threads.
unsafe impl Send for FilterContextSink<'_> {}

impl FilterContextSink<'_> {
    pub fn receive_frame(&mut self) -> Result<Option<Frame>, FfmpegError> {
        let mut frame = Frame::new()?;

        // Safety: `frame` is a valid pointer, and `self.0` is a valid pointer.
        unsafe {
            match av_buffersink_get_frame(self.0, frame.as_mut_ptr()) {
                0 => Ok(Some(frame)),
                AVERROR_EAGAIN | AVERROR_EOF => Ok(None),
                e => Err(FfmpegError::Code(e.into())),
            }
        }
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use std::ffi::CString;

    use ffmpeg_sys_next::avfilter_get_by_name;
    use ffmpeg_sys_next::AVSampleFormat::{AV_SAMPLE_FMT_FLTP, AV_SAMPLE_FMT_S16};

    use crate::filter_graph::{Filter, FilterGraph, FilterGraphParser};
    use crate::frame::Frame;

    #[test]
    fn test_filter_graph_new() {
        let filter_graph = FilterGraph::new();
        assert!(filter_graph.is_ok(), "FilterGraph::new should create a valid filter graph");

        if let Ok(graph) = filter_graph {
            assert!(!graph.as_ptr().is_null(), "FilterGraph pointer should not be null");
        }
    }

    #[test]
    fn test_filter_graph_as_mut_ptr() {
        let mut filter_graph = FilterGraph::new().expect("Failed to create filter graph");
        let raw_ptr = filter_graph.as_mut_ptr();

        assert!(!raw_ptr.is_null(), "FilterGraph::as_mut_ptr should return a valid pointer");
    }

    #[test]
    fn test_filter_graph_add() {
        let mut filter_graph = FilterGraph::new().expect("Failed to create filter graph");
        let filter_name = "buffer";
        let filter_ptr = unsafe { avfilter_get_by_name(CString::new(filter_name).unwrap().as_ptr()) };
        assert!(
            !filter_ptr.is_null(),
            "avfilter_get_by_name should return a valid pointer for filter '{}'",
            filter_name
        );

        let filter = unsafe { Filter::wrap(filter_ptr) };
        let name = "buffer_filter";
        let args = "width=1920:height=1080:pix_fmt=0:time_base=1/30";
        let result = filter_graph.add(filter, name, args);

        assert!(
            result.is_ok(),
            "FilterGraph::add should successfully add a filter to the graph"
        );

        if let Ok(context) = result {
            assert!(
                !context.0.filter.is_null(),
                "The filter context should have a valid filter pointer"
            );
        }
    }

    #[test]
    fn test_filter_graph_get() {
        let mut filter_graph = FilterGraph::new().expect("Failed to create filter graph");
        let filter_name = "buffer";
        let filter_ptr = unsafe { avfilter_get_by_name(CString::new(filter_name).unwrap().as_ptr()) };
        assert!(
            !filter_ptr.is_null(),
            "avfilter_get_by_name should return a valid pointer for filter '{}'",
            filter_name
        );

        let filter = unsafe { Filter::wrap(filter_ptr) };
        let name = "buffer_filter";
        let args = "width=1920:height=1080:pix_fmt=0:time_base=1/30";
        filter_graph
            .add(filter, name, args)
            .expect("Failed to add filter to the graph");

        let result = filter_graph.get(name);
        assert!(
            result.is_some(),
            "FilterGraph::get should return Some(FilterContext) for an existing filter"
        );

        if let Some(filter_context) = result {
            assert!(
                !filter_context.0.filter.is_null(),
                "The retrieved FilterContext should have a valid filter pointer"
            );
        }

        let non_existent = filter_graph.get("non_existent_filter");
        assert!(
            non_existent.is_none(),
            "FilterGraph::get should return None for a non-existent filter"
        );
    }

    #[test]
    fn test_filter_graph_validate_and_dump() {
        let mut filter_graph = FilterGraph::new().expect("Failed to create filter graph");
        let filter_spec = "anullsrc=sample_rate=44100:channel_layout=stereo [out0]; [out0] anullsink";
        FilterGraphParser::new(&mut filter_graph)
            .parse(filter_spec)
            .expect("Failed to parse filter graph spec");

        filter_graph.validate().expect("FilterGraph::validate should succeed");
        let dump_output = filter_graph.dump().expect("Failed to dump the filter graph");

        assert!(
            dump_output.contains("anullsrc"),
            "Dump output should include the 'anullsrc' filter type"
        );
        assert!(
            dump_output.contains("anullsink"),
            "Dump output should include the 'anullsink' filter type"
        );
    }

    #[test]
    fn test_filter_graph_set_thread_count() {
        let mut filter_graph = FilterGraph::new().expect("Failed to create filter graph");
        filter_graph.set_thread_count(4);
        assert_eq!(
            unsafe { (*filter_graph.as_mut_ptr()).nb_threads },
            4,
            "Thread count should be set to 4"
        );

        filter_graph.set_thread_count(8);
        assert_eq!(
            unsafe { (*filter_graph.as_mut_ptr()).nb_threads },
            8,
            "Thread count should be set to 8"
        );
    }

    #[test]
    fn test_filter_graph_input() {
        let mut filter_graph = FilterGraph::new().expect("Failed to create filter graph");
        let anullsrc = Filter::get("anullsrc").expect("Failed to get 'anullsrc' filter");
        filter_graph
            .add(anullsrc, "src", "sample_rate=44100:channel_layout=stereo")
            .expect("Failed to add 'anullsrc' filter");
        let input_parser = filter_graph
            .input("src", 0)
            .expect("Failed to set input for the filter graph");

        assert!(
            input_parser.graph.as_ptr() == filter_graph.as_ptr(),
            "Input parser should belong to the same filter graph"
        );
    }

    #[test]
    fn test_filter_graph_output() {
        let mut filter_graph = FilterGraph::new().expect("Failed to create filter graph");
        let anullsink = Filter::get("anullsink").expect("Failed to get 'anullsink' filter");
        filter_graph
            .add(anullsink, "sink", "")
            .expect("Failed to add 'anullsink' filter");
        let output_parser = filter_graph
            .output("sink", 0)
            .expect("Failed to set output for the filter graph");

        assert!(
            output_parser.graph.as_ptr() == filter_graph.as_ptr(),
            "Output parser should belong to the same filter graph"
        );
    }

    #[test]
    fn test_filter_context_source() {
        let mut filter_graph = FilterGraph::new().expect("Failed to create filter graph");
        let anullsrc = Filter::get("anullsrc").expect("Failed to get 'anullsrc' filter");
        filter_graph
            .add(anullsrc, "src", "sample_rate=44100:channel_layout=stereo")
            .expect("Failed to add 'anullsrc' filter");
        let filter_context = filter_graph.get("src").expect("Failed to retrieve 'src' filter context");
        let source_context = filter_context.source();

        assert!(
            std::ptr::eq(source_context.0, filter_graph.get("src").unwrap().0),
            "Source context should wrap the same filter as the original filter context"
        );
    }

    #[test]
    fn test_filter_context_sink() {
        let mut filter_graph = FilterGraph::new().expect("Failed to create filter graph");
        let anullsink = Filter::get("anullsink").expect("Failed to get 'anullsink' filter");
        filter_graph
            .add(anullsink, "sink", "")
            .expect("Failed to add 'anullsink' filter");
        let filter_context = filter_graph.get("sink").expect("Failed to retrieve 'sink' filter context");
        let sink_context = filter_context.sink();

        assert!(
            std::ptr::eq(sink_context.0, filter_graph.get("sink").unwrap().0),
            "Sink context should wrap the same filter as the original filter context"
        );
    }

    #[test]
    fn test_filter_context_source_send_and_receive_frame() {
        let mut filter_graph = FilterGraph::new().expect("Failed to create filter graph");
        let filter_spec = "\
            abuffer=sample_rate=44100:sample_fmt=s16:channel_layout=stereo:time_base=1/44100 \
            [out]; \
            [out] abuffersink";
        FilterGraphParser::new(&mut filter_graph)
            .parse(filter_spec)
            .expect("Failed to parse filter graph spec");
        filter_graph.validate().expect("Failed to validate filter graph");

        let source_context_name = "Parsed_abuffer_0";
        let sink_context_name = "Parsed_abuffersink_1";

        let mut frame = Frame::new().expect("Failed to create frame");
        frame.set_format(AV_SAMPLE_FMT_S16 as i32);
        let mut audio_frame = frame.audio();
        audio_frame.set_nb_samples(1024);
        audio_frame.set_sample_rate(44100);

        assert!(
            audio_frame.set_channel_layout_default(2).is_ok(),
            "Failed to set default channel layout"
        );
        assert!(
            audio_frame.alloc_frame_buffer(None).is_ok(),
            "Failed to allocate frame buffer"
        );

        let mut source_context = filter_graph
            .get(source_context_name)
            .expect("Failed to retrieve source filter context")
            .source();

        let result = source_context.send_frame(&audio_frame);
        assert!(result.is_ok(), "send_frame should succeed when sending a valid frame");

        let mut sink_context = filter_graph
            .get(sink_context_name)
            .expect("Failed to retrieve sink filter context")
            .sink();
        let received_frame = sink_context
            .receive_frame()
            .expect("Failed to receive frame from sink context");

        assert!(received_frame.is_some(), "No frame received from sink context");

        insta::assert_debug_snapshot!(received_frame.unwrap(), @r"
        Frame {
            pts: None,
            dts: None,
            duration: Some(
                1024,
            ),
            best_effort_timestamp: None,
            time_base: AVRational {
                num: 0,
                den: 1,
            },
            format: 1,
            is_audio: true,
            is_video: false,
        }
        ");
    }

    #[test]
    fn test_filter_context_source_send_frame_error() {
        let mut filter_graph = FilterGraph::new().expect("Failed to create filter graph");
        let filter_spec = "\
            abuffer=sample_rate=44100:sample_fmt=s16:channel_layout=stereo:time_base=1/44100 \
            [out]; \
            [out] anullsink";
        FilterGraphParser::new(&mut filter_graph)
            .parse(filter_spec)
            .expect("Failed to parse filter graph spec");
        filter_graph.validate().expect("Failed to validate filter graph");

        let mut source_context = filter_graph
            .get("Parsed_abuffer_0")
            .expect("Failed to retrieve 'Parsed_abuffer_0' filter context")
            .source();

        // create frame w/ mismatched format and sample rate
        let mut frame = Frame::new().expect("Failed to create frame");
        frame.set_format(AV_SAMPLE_FMT_FLTP as i32);
        let result = source_context.send_frame(&frame);

        assert!(result.is_err(), "send_frame should fail when sending an invalid frame");
    }

    #[test]
    fn test_filter_context_source_send_and_receive_eof() {
        let mut filter_graph = FilterGraph::new().expect("Failed to create filter graph");
        let filter_spec = "\
            abuffer=sample_rate=44100:sample_fmt=s16:channel_layout=stereo:time_base=1/44100 \
            [out]; \
            [out] abuffersink";
        FilterGraphParser::new(&mut filter_graph)
            .parse(filter_spec)
            .expect("Failed to parse filter graph spec");
        filter_graph.validate().expect("Failed to validate filter graph");

        let source_context_name = "Parsed_abuffer_0";
        let sink_context_name = "Parsed_abuffersink_1";

        {
            let mut source_context = filter_graph
                .get(source_context_name)
                .expect("Failed to retrieve source filter context")
                .source();
            let eof_result_with_pts = source_context.send_eof(Some(12345));
            assert!(eof_result_with_pts.is_ok(), "send_eof with PTS should succeed");

            let eof_result_without_pts = source_context.send_eof(None);
            assert!(eof_result_without_pts.is_ok(), "send_eof without PTS should succeed");
        }

        {
            let mut sink_context = filter_graph
                .get(sink_context_name)
                .expect("Failed to retrieve sink filter context")
                .sink();
            let received_frame = sink_context.receive_frame();
            assert!(received_frame.is_ok(), "receive_frame should succeed after EOF is sent");
            assert!(received_frame.unwrap().is_none(), "No frame should be received after EOF");
        }
    }
}
