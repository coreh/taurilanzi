import type { Direction } from '~/lib/key-grid'
import { neighbourKey } from '~/lib/key-grid'

/** A key of a page: what the inspector edits. */
export interface KeyRef { page: string, index: number }

const selection = ref<KeyRef | null>(null)
/** The line picked on the canvas, if the selection came from clicking one. */
const selectedLink = ref<string | null>(null)
/** A page the canvas should bring into view; the counter repeats requests for the same page. */
const focusRequest = ref<{ page: string, count: number } | null>(null)
/** The column the last arrow move went through, for leaving the wide key the way it was entered. */
let arrowColumn: { page: string, index: number, column: number } | null = null
/** A dialog of the inspector to open for a key: its type's settings, or the choice of its type. */
const keyRequest = shallowRef<KeyRef & { open: 'setup' | 'type' } | null>(null)

/** What the user is looking at and editing, shared by the canvas, the sidebar and the inspector. */
export function useWorkspace() {
  const { snapshot } = useEngine()
  const { removeKey } = usePages()

  const pages = computed(() => snapshot.value?.pages ?? [])
  const current = computed(() => snapshot.value?.path.at(-1) ?? null)

  function select(page: string, index: number) {
    selection.value = { page, index }
    selectedLink.value = null
  }

  /** Picking a line also selects its key, whose actions the inspector shows. */
  function selectLink(page: string, index: number) {
    selection.value = { page, index }
    selectedLink.value = `${page}:${index}`
  }

  function clear() {
    selection.value = null
    selectedLink.value = null
  }

  /** Selects the key next to the selected one, as the inspector would after a click. */
  function moveSelection(direction: Direction) {
    const from = selection.value
    if (!from) {
      return false
    }
    const remembered = arrowColumn?.page === from.page && arrowColumn.index === from.index ? arrowColumn.column : undefined
    const next = neighbourKey(from.index, direction, remembered)
    if (!next) {
      return false
    }
    select(from.page, next.index)
    arrowColumn = { page: from.page, index: next.index, column: next.column }
    return true
  }

  /** Deletes the selected key as its context menu does; undo brings it back. */
  async function removeSelectedKey() {
    const target = selection.value
    if (!target || !snapshot.value?.pages.find(p => p.id === target.page)?.keys[target.index]) {
      return
    }
    // Its line goes with it, so a selection that came from the line goes too.
    if (selectedLink.value) {
      clear()
    }
    await removeKey(target.page, target.index)
  }

  /** Selects the key and opens what sets it up: the settings of its type, or the choice of one
   * on an empty key. */
  function setUpKey(page: string, index: number) {
    select(page, index)
    keyRequest.value = { page, index, open: 'setup' }
  }

  function changeKeyType(page: string, index: number) {
    select(page, index)
    keyRequest.value = { page, index, open: 'type' }
  }

  function focus(page: string) {
    focusRequest.value = { page, count: (focusRequest.value?.count ?? 0) + 1 }
  }

  watch(pages, (list) => {
    if (selection.value && !list.some(p => p.id === selection.value!.page)) {
      selection.value = null
    }
  })

  return { selection, selectedLink, focusRequest, keyRequest, pages, current, select, selectLink, clear, focus, moveSelection, removeSelectedKey, setUpKey, changeKeyType }
}
