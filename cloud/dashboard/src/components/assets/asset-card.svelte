<script lang="ts">
    import IconTimelineDot from "$lib/images/icon-timeline-dot.svelte";
    import IconWebhook from "$lib/images/icon-webhook.svelte";
    import StreamStatusPill from "$lib/shared-components/stream-status-pill.svelte";

    let {
        status = "live",
        streamName = "unique-stream-name",
        streamId = "8a28e499d6s7987fd981293fd981293",
        startTime = "May 5, 04:01:11 PM EDT",
        duration = "1h 30min",
        endTime = "May 6, 04:01:11 PM EDT",
    } = $props();

    const isLive = $derived(status === "live");

    // const statusIcon = {
    //     live: IconRoadmapDot,
    //     finished: IconRoadmapDot,
    // };

    // const IconComponent = statusIcon[status as keyof typeof statusIcon];
</script>

<div class="stream-card">
    <div class="left-section">
        <div class="status-section">
            <IconWebhook />
            <StreamStatusPill status="live" />
            <h3 class="stream-name">{streamName}</h3>
        </div>
        <div class="stream-details">
            <p class="stream-id">{streamId}</p>
        </div>
    </div>
    <div class="right-section">
        <div class="start-time">
            <span class="time">{startTime}</span>
            <IconTimelineDot variant="default" />
        </div>
        <div class="time-divider"></div>
        <div class="end-time">
            <span class="duration">{duration}</span>
            <span class="time">{endTime}</span>
            <IconTimelineDot variant="finished" />
        </div>
    </div>
</div>

<style>
    .stream-card {
      background: var(--colors-gray20);
      border-radius: 0.75rem;
      padding: 0.75rem;
      display: flex;
      align-items: center;
      justify-content: space-between;
      cursor: pointer;
      transition: background-color 0.2s ease;
    }

    .left-section {
      display: flex;
      flex-direction: column;
      gap: 1rem;

      .status-section {
        display: flex;
        flex-direction: row;
        gap: 0.5rem;
        align-items: center;

        .stream-name {
          color: white;
          font-weight: 500;
          font-size: 0.875rem;
          margin: 0;
        }
      }
    }

    .stream-details {
      display: flex;
      flex-direction: column;
    }

    .stream-id {
      color: #9ca3af;
      font-size: 0.75rem;
      margin: 0;
    }

    .right-section {
      display: flex;
      flex-direction: column;
      align-items: flex-end;
      text-align: right;
      justify-content: center;

      .start-time {
        font-size: 0.875rem;
        font-style: normal;
        font-weight: 600;
        line-height: 1rem;
        color: #282625;
        display: flex;
        gap: 0.5rem;
      }

      .time-divider {
        width: 0.125rem;
        align-self: right;
        height: 1rem;
        background-color: var(--colors-gray50);
        margin-right: 0.3rem;
      }

      .end-time {
        font-size: 0.875rem;
        font-style: normal;
        font-weight: 600;
        line-height: 1rem;
        color: #282625;
        display: flex;
        gap: 0.5rem;
      }
    }
</style>
