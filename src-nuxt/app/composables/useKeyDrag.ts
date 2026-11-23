import { commands } from '~/ipc/bindings'
import type { KeyRef } from '~/composables/useWorkspace'
import { os } from '~/lib/os'

/** Pixels the pointer must travel before a press on a key becomes a drag. */
const THRESHOLD = 5

interface Drag {
  from: KeyRef
  image: string | null
  wide: boolean
  x: number
  y: number
  over: KeyRef | null
  /** Dropping leaves the key where it was and puts a copy at the destination. */
  copy: boolean
}

const drag = shallowRef<Drag | null>(null)
let pending: { from: KeyRef, image: string | null, wide: boolean, x: number, y: number } | null = null
/** A drag ends with a click on the key it started from; that click must not select it. */
let swallowClick = false

/** Option on a Mac, as Finder copies; Ctrl elsewhere, where Alt-dragging moves windows. */
function copying(event: { altKey: boolean, ctrlKey: boolean }) {
  return os === 'macos' ? event.altKey : event.ctrlKey
}

/** The modifier can be pressed or let go while the key is held still. */
function modifierChanged(event: KeyboardEvent) {
  if (drag.value && drag.value.copy !== copying(event)) {
    drag.value = { ...drag.value, copy: copying(event) }
  }
}

function listenToModifier(on: boolean) {
  for (const type of ['keydown', 'keyup'] as const) {
    if (on) {
      window.addEventListener(type, modifierChanged)
    }
    else {
      window.removeEventListener(type, modifierChanged)
    }
  }
}

function slotAt(x: number, y: number): KeyRef | null {
  const el = document.elementFromPoint(x, y)?.closest<HTMLElement>('[data-key-slot]')
  if (!el?.dataset.page || el.dataset.index === undefined) {
    return null
  }
  return { page: el.dataset.page, index: Number(el.dataset.index) }
}

/** Moving keys by dragging them onto another place of any page on the canvas, or copying them
 * there with the modifier held. */
export function useKeyDrag() {

  function press(event: PointerEvent, from: KeyRef, image: string | null, wide: boolean) {
    if (event.button !== 0 || !image) {
      return
    }
    pending = { from, image, wide, x: event.clientX, y: event.clientY }
    ;(event.currentTarget as HTMLElement).setPointerCapture(event.pointerId)
  }

  function move(event: PointerEvent) {
    if (!pending && !drag.value) {
      return
    }
    if (pending && Math.hypot(event.clientX - pending.x, event.clientY - pending.y) < THRESHOLD) {
      return
    }
    const from = drag.value ?? pending!
    if (!drag.value) {
      listenToModifier(true)
    }
    pending = null
    drag.value = {
      from: from.from,
      image: from.image,
      wide: from.wide,
      x: event.clientX,
      y: event.clientY,
      over: slotAt(event.clientX, event.clientY),
      copy: copying(event),
    }
  }

  async function release(event?: PointerEvent) {
    pending = null
    const done = drag.value
    if (!done) {
      return
    }
    drag.value = null
    listenToModifier(false)
    swallowClick = true
    const to = done.over
    if (!to || (to.page === done.from.page && to.index === done.from.index)) {
      return
    }
    // A cancelled pointer says nothing of the keys; the last thing seen of them stands.
    const copy = event?.type === 'pointerup' ? copying(event) : done.copy
    await call(copy ? commands.copyKey(done.from, to) : commands.moveKey(done.from, to))
  }

  /** True once for the click that follows a drag. */
  function consumeClick() {
    const swallow = swallowClick
    swallowClick = false
    return swallow
  }

  function isOver(slot: KeyRef) {
    const over = drag.value?.over
    return Boolean(over && over.page === slot.page && over.index === slot.index)
  }

  function isDragging(slot: KeyRef) {
    const from = drag.value?.from
    return Boolean(from && from.page === slot.page && from.index === slot.index)
  }

  function isCopying() {
    return Boolean(drag.value?.copy)
  }

  return { drag, press, move, release, consumeClick, isOver, isDragging, isCopying }
}
