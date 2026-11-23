import { commands } from '~/ipc/bindings'
import { os } from '~/lib/os'

/** Editors with changes not saved yet, and whether each has some now; undo drops those first,
 * as they are the latest. */
const pending = shallowReactive(new Map<() => boolean, () => boolean>())
let listening = false

export function registerPendingEdit(discard: () => boolean, isPending: () => boolean = () => false) {
  pending.set(discard, isPending)
  return () => pending.delete(discard)
}

const hasPending = computed(() => [...pending.values()].some(isPending => isPending()))

async function undo() {
  for (const discard of pending.keys()) {
    if (discard()) {
      return
    }
  }
  await call(commands.undo())
}

async function redo() {
  await call(commands.redo())
}

export function typingIn(element: EventTarget | null) {
  return element instanceof HTMLTextAreaElement
    || (element instanceof HTMLInputElement && !['checkbox', 'radio', 'range', 'color', 'file', 'button'].includes(element.type))
    || (element instanceof HTMLElement && element.isContentEditable)
}

/** Ctrl+Z, Ctrl+Shift+Z and Ctrl+Y, as Windows and Linux apps take them. */
function historyKey(event: KeyboardEvent): 'undo' | 'redo' | null {
  if (!event.ctrlKey || event.altKey || event.metaKey) {
    return null
  }
  if (event.code === 'KeyZ') {
    return event.shiftKey ? 'redo' : 'undo'
  }
  return event.code === 'KeyY' && !event.shiftKey ? 'redo' : null
}

/** Undo and redo of edits. Undoing inside a text field undoes the typing, as anywhere else. */
export function useHistory() {
  const { snapshot } = useEngine()

  // On macOS the menu bar takes the keys (useAppMenu).
  if (!listening && import.meta.client && os !== 'macos') {
    listening = true
    // Without a menu bar the window gets the keys itself; text fields keep their own undo.
    window.addEventListener('keydown', (event) => {
      const wanted = historyKey(event)
      if (wanted === null || typingIn(document.activeElement)) {
        return
      }
      event.preventDefault()
      ;(wanted === 'redo' ? redo() : undo()).catch(() => {})
    })
  }

  return {
    canUndo: computed(() => snapshot.value?.can_undo ?? false),
    canRedo: computed(() => snapshot.value?.can_redo ?? false),
    hasPending,
    undo,
    redo,
  }
}
