import type { DeepReadonly } from 'vue'
import { commands } from '~/ipc/bindings'
import type { Action, Key, Page, Position } from '~/types/engine'
import { focusPage } from '~/composables/useWorkspace'
import type { KeyRef } from '~/composables/useWorkspace'
import type { NewKeyStyle } from '~/lib/new-key'
import { newKeyAppearance } from '~/lib/new-key'
import { t } from '~/lib/i18n'
import { kindOf } from '~/composables/useKeyEditor'

/** A page card on the canvas and the space left after it. */
const CARD_WIDTH = 320
const CARD_GAP = 80
/** Time for a new page to be measured before the canvas can bring it into view. */
const MEASURE_MS = 100

const NAVIGATION = new Set<Action['type']>(['open_page', 'back', 'home', 'previous_page', 'next_page', 'switch_profile', 'previous_profile', 'next_profile'])

export function isNavigation(action: Action) {
  return NAVIGATION.has(action.type)
}

/** The line drawn on the canvas replaces the key's navigation, or goes first if it had none:
 * right after what the key is, on a key whose first action is its type. */
export function withLink(actions: Action[], target: string, afterFirst = false): Action[] {
  const link: Action = { type: 'open_page', page: target }
  const at = actions.findIndex(isNavigation)
  if (at !== -1) {
    return actions.map((a, i) => (i === at ? link : a))
  }
  return afterFirst ? [actions[0]!, link, ...actions.slice(1)] : [link, ...actions]
}

/** Page and line edits from the canvas, saved straight away. */
export function usePages() {
  const { snapshot, refresh } = useEngine()

  function page(id: string): DeepReadonly<Page> | undefined {
    return snapshot.value?.pages.find(p => p.id === id)
  }

  function newKeyStyle() {
    return snapshot.value!.preferences.new_keys as NewKeyStyle
  }

  /** New pages go to the right of the rightmost one, so they never cover another. */
  function nextPosition(): Position {
    const list = snapshot.value?.pages ?? []
    const right = list.reduce<DeepReadonly<Page> | undefined>((max, p) => (!max || p.position.x > max.position.x ? p : max), undefined)
    return right ? { x: right.position.x + CARD_WIDTH + CARD_GAP, y: right.position.y } : { x: 0, y: 0 }
  }

  async function create(position: Position, name?: string) {
    return await call(commands.createPage(name ?? null, position))
  }

  async function update(id: string, changes: { name?: string, position?: Position }) {
    await call(commands.updatePage(id, { name: changes.name ?? null, position: changes.position ?? null }))
  }

  async function duplicate(id: string) {
    return await call(commands.duplicatePage(id))
  }

  async function remove(id: string) {
    await call(commands.deletePage(id))
  }

  async function setHome(id: string) {
    await call(commands.setHomePage(id))
  }

  async function show(id: string) {
    await call(commands.showPage(id))
  }

  async function saveKey(pageId: string, index: number, key: Key) {
    await call(commands.setKey(pageId, index, key))
  }

  /** An empty key gets the look of new keys with the target's name, so the new line is visible. */
  async function link(pageId: string, index: number, target: string) {
    const existing = page(pageId)?.keys[index]
    let key: Key
    if (existing) {
      const { kind, actions } = kindOf(existing)
      key = { ...(structuredClone(toRaw(existing)) as Key), kind, actions: withLink(JSON.parse(JSON.stringify(actions)) as Action[], target, kind === 'action') }
    }
    else {
      key = {
        kind: 'action',
        appearance: newKeyAppearance(newKeyStyle(), page(target)?.name ?? t('editor.newKey.page')),
        actions: [{ type: 'open_page', page: target }],
        double_press: [],
        long_press: [],
        second_state: null,
      }
    }
    await saveKey(pageId, index, key)
  }

  /** Replaces whatever is at `to` with a copy of the key at `from`. */
  async function copyKey(from: KeyRef, to: KeyRef) {
    await call(commands.copyKey(from, to))
  }

  /** Copies a key to the nearest free place of its page; a full page sends it to a new page,
   * which the canvas then brings into view. Answers where the copy went. */
  async function duplicateKey(from: KeyRef) {
    const to = await call(commands.duplicateKey(from, nextPosition()))
    if (to.page !== from.page) {
      await refresh()
      setTimeout(() => focusPage(to.page), MEASURE_MS)
    }
    return to
  }

  /** Deleting a key takes its line with it, and the other way round. */
  async function removeKey(pageId: string, index: number) {
    await call(commands.clearKey(pageId, index))
  }

  /** Saves the points the user placed on the line of a key. */
  async function setWaypoints(pageId: string, index: number, waypoints: Position[]) {
    const existing = page(pageId)?.keys[index]
    if (!existing) {
      return
    }
    const rounded = waypoints.map(p => ({ x: Math.round(p.x), y: Math.round(p.y) }))
    const actions = (existing.actions as Action[]).map(a => (a.type === 'open_page' ? { ...a, waypoints: rounded } : a))
    await saveKey(pageId, index, { ...(structuredClone(toRaw(existing)) as Key), actions })
  }

  return { page, nextPosition, create, update, duplicate, remove, setHome, show, link, copyKey, duplicateKey, removeKey, setWaypoints }
}
