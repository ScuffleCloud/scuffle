use std::{
    borrow::Cow, cmp::Reverse, collections::{BinaryHeap, HashMap}, fmt::Write, hash::BuildHasher, rc::Rc
};

use pulldown_cmark::{BrokenLink, CowStr, Event, Options, Tag};
use rustdoc_types::{
    Crate, Id, Item, ItemEnum, ItemKind, ItemSummary, MacroKind, StructKind, VariantKind,
};

trait CowStrExt<'a> {
    fn as_str(&'a self) -> &'a str;
}

impl<'a> CowStrExt<'a> for CowStr<'a> {
    fn as_str(&'a self) -> &'a str {
        match self {
            CowStr::Boxed(s) => s,
            CowStr::Borrowed(s) => s,
            CowStr::Inlined(s) => s,
        }
    }
}

#[derive(Debug)]
pub(super) struct Parser<B, M> {
    broken_link_callback: B,
    iterator_map: M,
}

type BrokenLinkPair<'a> = (CowStr<'a>, CowStr<'a>);

impl Parser<(), ()> {
    pub(super) fn new<'a>(
        doc: &'a Crate,
        item: &'a Item,
        local_html_root_url: &str,
        mappings: &HashMap<String, String>,
    ) -> Parser<
        impl FnMut(BrokenLink<'_>) -> Option<BrokenLinkPair<'a>>,
        impl FnMut(Event<'a>) -> Option<Event<'a>>,
    > {
        let url_map = Rc::new(resolve_links(doc, item, local_html_root_url, mappings));

        let broken_link_callback = {
            let url_map = Rc::clone(&url_map);
            move |link: BrokenLink<'_>| {
                let url = url_map.get(link.reference.as_str())?.as_ref()?;
                Some((url.to_owned().into(), "".into()))
            }
        };
        let iterator_map = move |event| convert_link(&url_map, event);

        Parser {
            broken_link_callback,
            iterator_map,
        }
    }
}

impl<'a, B, M> Parser<B, M>
where
    B: FnMut(BrokenLink<'_>) -> Option<BrokenLinkPair<'a>> + 'a,
    M: FnMut(Event<'a>) -> Option<Event<'a>> + 'a,
{
    pub(super) fn events<'b>(&'b mut self, doc: &'a str) -> impl Iterator<Item = Event<'a>> + 'b
    where
        'a: 'b,
    {
        pulldown_cmark::Parser::new_with_broken_link_callback(
            doc,
            Options::all(),
            Some(&mut self.broken_link_callback),
        )
        .filter_map(&mut self.iterator_map)
    }
}

fn resolve_links<'doc>(
    doc: &'doc Crate,
    item: &'doc Item,
    local_html_root_url: &str,
    mappings: &HashMap<String, String>,
) -> HashMap<&'doc str, Option<String>> {
    let extra_paths = extra_paths(&doc.index, &doc.paths);
    item.links
        .iter()
        .map(move |(name, id)| {
            if let Some(path) = mappings.get(name) {
                (name.as_str(), Some(path.clone()))
            } else {
                let url = id_to_url(doc, &extra_paths, local_html_root_url, id).or_else(|| {
                    tracing::warn!(?id, "failed to resolve link to `{name}`");
                    None
                });
                (name.as_str(), url)
            }
        })
        .collect()
}

#[derive(Debug)]
struct Node<'a> {
    depth: usize,
    kind: ItemKind,
    name: Option<&'a str>,
    parent: Option<&'a Id>,
}

fn extra_paths<'doc, S: BuildHasher + Default>(
    index: &'doc HashMap<Id, Item, S>,
    paths: &'doc HashMap<Id, ItemSummary, S>,
) -> HashMap<&'doc Id, Node<'doc>, S> {
    let mut map: HashMap<&Id, Node<'_>, S> = index
        .iter()
        .map(|(id, item)| {
            (
                id,
                Node {
                    depth: usize::MAX,
                    kind: item_kind(item),
                    name: item.name.as_deref(),
                    parent: None,
                },
            )
        })
        .collect();

    #[derive(Debug)]
    struct HeapItem<'doc> {
        depth: Reverse<usize>,
        id: &'doc Id,
        parent: Option<&'doc Id>,
        item: &'doc Item,
    }

    impl PartialOrd for HeapItem<'_> {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            Some(self.cmp(other))
        }
    }
    impl Ord for HeapItem<'_> {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            self.depth.cmp(&other.depth)
        }
    }
    impl PartialEq for HeapItem<'_> {
        fn eq(&self, other: &Self) -> bool {
            self.depth == other.depth
        }
    }
    impl Eq for HeapItem<'_> {}

    let mut heap: BinaryHeap<HeapItem<'_>> = index
        .iter()
        .map(|(id, item)| {
            let depth = if paths.contains_key(id) {
                0
            } else {
                usize::MAX
            };
            HeapItem {
                depth: Reverse(depth),
                id,
                item,
                parent: None,
            }
        })
        .collect();

    while let Some(HeapItem {
        depth: Reverse(depth),
        id,
        parent,
        item,
    }) = heap.pop()
    {
        let node = map.get_mut(id).unwrap();
        if depth >= node.depth {
            continue;
        }
        node.parent = parent;

        map.get_mut(id).unwrap().depth = depth;

        for child in item_children(item).into_iter().flatten() {
            let child = match index.get(child) {
                Some(child) => child,
                None => {
                    tracing::trace!(?item, ?child, "child item missing");
                    continue;
                }
            };
            let child_depth = depth + 1;
            heap.push(HeapItem {
                depth: Reverse(child_depth),
                id: &child.id,
                item: child,
                parent: Some(id),
            });
        }
    }

    map
}

fn item_kind(item: &Item) -> ItemKind {
    match &item.inner {
        ItemEnum::Module(_) => ItemKind::Module,
        ItemEnum::ExternCrate { .. } => ItemKind::ExternCrate,
        ItemEnum::Use(_) => ItemKind::Use,
        ItemEnum::Union(_) => ItemKind::Union,
        ItemEnum::Struct(_) => ItemKind::Struct,
        ItemEnum::StructField(_) => ItemKind::StructField,
        ItemEnum::Enum(_) => ItemKind::Enum,
        ItemEnum::Variant(_) => ItemKind::Variant,
        ItemEnum::Function(_) => ItemKind::Function,
        ItemEnum::Trait(_) => ItemKind::Trait,
        ItemEnum::TraitAlias(_) => ItemKind::TraitAlias,
        ItemEnum::Impl(_) => ItemKind::Impl,
        ItemEnum::TypeAlias(_) => ItemKind::TypeAlias,
        ItemEnum::Constant { .. } => ItemKind::Constant,
        ItemEnum::Static(_) => ItemKind::Static,
        ItemEnum::ExternType => ItemKind::ExternType,
        ItemEnum::Macro(_) => ItemKind::Macro,
        ItemEnum::ProcMacro(pm) => match pm.kind {
            MacroKind::Bang => ItemKind::Macro,
            MacroKind::Attr => ItemKind::ProcAttribute,
            MacroKind::Derive => ItemKind::ProcDerive,
        },
        ItemEnum::Primitive(_) => ItemKind::Primitive,
        ItemEnum::AssocConst { .. } => ItemKind::AssocConst,
        ItemEnum::AssocType { .. } => ItemKind::AssocType,
    }
}

fn item_children<'doc>(parent: &'doc Item) -> Option<Box<dyn Iterator<Item = &'doc Id> + 'doc>> {
    match &parent.inner {
        ItemEnum::Module(m) => Some(Box::new(m.items.iter())),
        ItemEnum::ExternCrate { .. } => None,
        ItemEnum::Use(_) => None,
        ItemEnum::Union(u) => Some(Box::new(u.fields.iter())),
        ItemEnum::Struct(s) => match &s.kind {
            StructKind::Unit => None,
            StructKind::Tuple(t) => Some(Box::new(t.iter().flatten())),
            StructKind::Plain {
                fields,
                has_stripped_fields: _,
            } => Some(Box::new(fields.iter())),
        },
        ItemEnum::StructField(_) => None,
        ItemEnum::Enum(e) => Some(Box::new(e.variants.iter())),
        ItemEnum::Variant(v) => match &v.kind {
            VariantKind::Plain => None,
            VariantKind::Tuple(t) => Some(Box::new(t.iter().flatten())),
            VariantKind::Struct {
                fields,
                has_stripped_fields: _,
            } => Some(Box::new(fields.iter())),
        },
        ItemEnum::Function(_) => None,
        ItemEnum::Trait(t) => Some(Box::new(t.items.iter())),
        ItemEnum::TraitAlias(_) => None,
        ItemEnum::Impl(i) => Some(Box::new(i.items.iter())),
        ItemEnum::TypeAlias(_) => None,
        ItemEnum::Constant { .. } => None,
        ItemEnum::Static(_) => None,
        ItemEnum::ExternType => None,
        ItemEnum::Macro(_) => None,
        ItemEnum::ProcMacro(_) => None,
        ItemEnum::Primitive(_) => None,
        ItemEnum::AssocConst { .. } => None,
        ItemEnum::AssocType { .. } => None,
    }
}

fn convert_link<'a>(
    url_map: &HashMap<&str, Option<String>>,
    mut event: Event<'a>,
) -> Option<Event<'a>> {
    if let Event::Start(Tag::Link { dest_url: url, .. }) = &mut event {
        if let Some(full_url) = url_map.get(url.as_ref()) {
            match full_url {
                Some(full_url) => *url = full_url.to_owned().into(),
                None => return None,
            }
        }
    }
    Some(event)
}

fn id_to_url<S: BuildHasher + Default>(
    doc: &Crate,
    extra_paths: &HashMap<&Id, Node<'_>, S>,
    local_html_root_url: &str,
    id: &Id,
) -> Option<String> {
    let item = item_summary(doc, extra_paths, id)?;
    let html_root_url = if item.crate_id == 0 {
        // local item
        local_html_root_url
    } else {
        // external item
        let external_crate = doc.external_crates.get(&item.crate_id)?;
        external_crate.html_root_url.as_ref()?
    };

    let mut url = html_root_url.trim_end_matches('/').to_owned();
    let mut join = |paths: &[String], args| {
        for path in paths {
            write!(&mut url, "/{path}").unwrap();
        }
        write!(&mut url, "/{args}").unwrap();
    };
    match (&item.kind, item.path.as_slice()) {
        (ItemKind::Module, ps) => join(ps, format_args!("index.html")),
        // (ItemKind::ExternCrate, [..]) => todo!(),
        // (ItemKind::Import, [..]) => todo!(),
        (ItemKind::Struct, [ps @ .., name]) => join(ps, format_args!("struct.{name}.html")),
        (ItemKind::StructField, [ps @ .., struct_name, field]) => join(
            ps,
            format_args!("struct.{struct_name}.html#structfield.{field}"),
        ),
        (ItemKind::Union, [ps @ .., name]) => join(ps, format_args!("union.{name}.html")),
        (ItemKind::Enum, [ps @ .., name]) => join(ps, format_args!("enum.{name}.html")),
        (ItemKind::Variant, [ps @ .., enum_name, variant_name]) => join(
            ps,
            format_args!("enum.{enum_name}.html#variant.{variant_name}"),
        ),
        (ItemKind::Function, [ps @ .., name]) => join(ps, format_args!("fn.{name}.html")),
        (ItemKind::TypeAlias, [ps @ .., name]) => join(ps, format_args!("type.{name}.html")),
        // (ItemKind::OpaqueTy, [..]) => todo!(),
        (ItemKind::Constant, [ps @ .., name]) => join(ps, format_args!("constant.{name}.html")),
        (ItemKind::Trait, [ps @ .., name]) => join(ps, format_args!("trait.{name}.html")),
        // (ItemKind::TraitAlias, [..]) => todo!(),
        // (ItemKind::Impl, [..]) => todo!(),
        (ItemKind::Static, [ps @ .., name]) => join(ps, format_args!("static.{name}.html")),
        // (ItemKind::ForeignType, [..]) => todo!(),
        (ItemKind::Macro, [ps @ .., name]) => join(ps, format_args!("macro.{name}.html")),
        (ItemKind::ProcAttribute, [ps @ .., name]) => join(ps, format_args!("attr.{name}.html")),
        (ItemKind::ProcDerive, [ps @ .., name]) => join(ps, format_args!("derive.{name}.html")),
        (ItemKind::AssocConst, [ps @ .., trait_name, const_name]) => join(
            ps,
            format_args!("trait.{trait_name}.html#associatedconstant.{const_name}"),
        ),
        (ItemKind::AssocType, [ps @ .., trait_name, type_name]) => join(
            ps,
            format_args!("trait.{trait_name}.html#associatedtype.{type_name}"),
        ),
        (ItemKind::Primitive, [ps @ .., name]) => join(ps, format_args!("primitive.{name}.html")),
        // (ItemKind::Keyword, [..]) => todo!(),
        (item, path) => {
            tracing::warn!(?item, ?path, "unexpected intra-doc link item & path found");
            return None;
        }
    }
    Some(url)
}

fn item_summary<'doc, S: BuildHasher + Default>(
    doc: &'doc Crate,
    extra_paths: &'doc HashMap<&'doc Id, Node<'doc>, S>,
    id: &'doc Id,
) -> Option<Cow<'doc, ItemSummary>> {
    if let Some(summary) = doc.paths.get(id) {
        return Some(Cow::Borrowed(summary));
    }
    // workaround for https://github.com/rust-lang/rust/issues/101687
    // if the item is not found in the paths, try to find it in the extra_paths

    let node = extra_paths.get(id)?;
    let mut stack = vec![node];
    let mut current = node;
    while let Some(parent) = current.parent {
        if let Some(summary) = doc.paths.get(parent) {
            let mut path = summary.path.clone();
            while let Some(node) = stack.pop() {
                let name = node.name?;
                path.push(name.to_string());
            }
            return Some(Cow::Owned(ItemSummary {
                crate_id: summary.crate_id,
                kind: node.kind,
                path,
            }));
        }
        current = extra_paths.get(&parent)?;
        stack.push(current);
    }
    None
}
