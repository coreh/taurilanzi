import { useVueFlow } from '@vue-flow/core'

/** Pixels a scroll by lines (`deltaMode` 1) moves, as Vue Flow counts them. */
const LINE_PX = 20
/** Wheel pixels that zoom by a factor of e: a trackpad pinch then follows the fingers. */
const ZOOM_PX = 100
/** Mouse wheels move 40 to 100 px a notch: capped, each notch zooms about 14 %. */
const MAX_ZOOM_PX = 15

/** Safari's pinch event, which TypeScript's DOM types leave out. */
interface GestureEvent extends UIEvent {
  scale: number
  clientX: number
  clientY: number
}

/**
 * Scrolling pans the canvas in any direction and pinching or Cmd/Ctrl + scroll zooms around the
 * pointer, as in Figma. Vue Flow's own `panOnScroll` falls short in the app's webviews: WKWebView
 * reports a trackpad pinch as Safari gesture events rather than the Ctrl + wheel events Vue Flow
 * listens for, and outside macOS Vue Flow pans with the Ctrl + wheel events a pinch sends.
 */
export function useCanvasGestures(flow: string, area: Ref<HTMLElement | null>) {
  const { d3Zoom, d3Selection, viewport, viewportRef, userSelectionActive, panBy } = useVueFlow(flow)
  /** The zoom when a Safari pinch began, which its events scale. */
  let pinchFrom: number | null = null

  function zoomAt(zoom: number, clientX: number, clientY: number) {
    const box = viewportRef.value?.getBoundingClientRect()
    if (d3Zoom.value && d3Selection.value && box) {
      d3Zoom.value.scaleTo(d3Selection.value, zoom, [clientX - box.left, clientY - box.top])
    }
  }

  function pixels(event: WheelEvent, delta: number) {
    if (event.deltaMode === WheelEvent.DOM_DELTA_LINE) {
      return delta * LINE_PX
    }
    if (event.deltaMode === WheelEvent.DOM_DELTA_PAGE) {
      return delta * (viewportRef.value?.clientHeight ?? 0)
    }
    return delta
  }

  function wheel(event: WheelEvent) {
    // Elements that scroll by themselves keep their wheel, as Vue Flow does for them.
    if (event.target instanceof Element && event.target.closest('.nowheel')) {
      return
    }
    event.preventDefault()
    event.stopPropagation()
    if (userSelectionActive.value) {
      return
    }
    if (event.ctrlKey || event.metaKey) {
      // Some webviews send a pinch both ways; the gesture events are the smoother ones.
      if (pinchFrom !== null) {
        return
      }
      const dy = Math.max(-MAX_ZOOM_PX, Math.min(MAX_ZOOM_PX, pixels(event, event.deltaY)))
      zoomAt(viewport.value.zoom * Math.exp(-dy / ZOOM_PX), event.clientX, event.clientY)
      return
    }
    let dx = pixels(event, event.deltaX)
    let dy = pixels(event, event.deltaY)
    // Shift + mouse wheel scrolls sideways where the system does not turn it by itself.
    if (event.shiftKey && !dx) {
      [dx, dy] = [dy, 0]
    }
    panBy({ x: -dx, y: -dy })
  }

  function gestureStart(event: Event) {
    event.preventDefault()
    pinchFrom = viewport.value.zoom
  }

  function gestureChange(event: Event) {
    event.preventDefault()
    const gesture = event as GestureEvent
    if (pinchFrom !== null && !userSelectionActive.value) {
      zoomAt(pinchFrom * gesture.scale, gesture.clientX, gesture.clientY)
    }
  }

  function gestureEnd(event: Event) {
    event.preventDefault()
    pinchFrom = null
  }

  const listeners: Record<string, EventListener> = {
    wheel: event => wheel(event as WheelEvent),
    gesturestart: gestureStart,
    gesturechange: gestureChange,
    gestureend: gestureEnd,
  }
  // Captured before Vue Flow's own listeners, which would also zoom with a plain wheel.
  onMounted(() => {
    for (const [type, listener] of Object.entries(listeners)) {
      area.value?.addEventListener(type, listener, { capture: true, passive: false })
    }
  })
  onBeforeUnmount(() => {
    for (const [type, listener] of Object.entries(listeners)) {
      area.value?.removeEventListener(type, listener, { capture: true })
    }
  })
}
