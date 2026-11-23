import { commands, events } from '~/ipc/bindings'
import type { MenuContext } from '~/ipc/bindings'
import { os } from '~/lib/os'
import { typingIn } from '~/composables/useHistory'

type Run = () => unknown

/** What each command of the menu bar does, set by whoever owns what it acts on. */
const handlers = new Map<string, Run>()
let listening = false

/** Carries out a command of the macOS menu bar (`file.newPage`…) while the calling component is
 * mounted. The menu bar's item comes from `src-tauri/src/app_menu/layout.rs`. */
export function useMenuCommand(command: string, run: Run) {
  onMounted(() => handlers.set(command, run))
  onBeforeUnmount(() => {
    if (handlers.get(command) === run) {
      handlers.delete(command)
    }
  })
}

/** A command nobody handles is of a view that is not on screen, whose items are disabled. */
function dispatch(command: string) {
  const run = handlers.get(command)
  if (run) {
    Promise.resolve(run()).catch(() => {})
  }
}

/** The macOS menu bar: carries out its commands and tells it what can be done now. Used once, by
 * the layout. */
export function useAppMenu() {
  if (os !== 'macos' || !import.meta.client) {
    return
  }
  const history = useHistory()
  const route = useRoute()
  const { selection } = useWorkspace()
  const plugins = usePlugins()

  const typing = ref(typingIn(document.activeElement))
  const focusIn = (event: FocusEvent) => { typing.value = typingIn(event.target) }
  const focusOut = (event: FocusEvent) => { typing.value = typingIn(event.relatedTarget) }
  onMounted(() => {
    document.addEventListener('focusin', focusIn)
    document.addEventListener('focusout', focusOut)
  })
  onBeforeUnmount(() => {
    document.removeEventListener('focusin', focusIn)
    document.removeEventListener('focusout', focusOut)
  })

  /** Inside a text field, undo is the field's own, as anywhere else. */
  function historyStep(redo: boolean) {
    if (typingIn(document.activeElement)) {
      document.execCommand(redo ? 'redo' : 'undo')
      return
    }
    return redo ? history.redo() : history.undo()
  }

  useMenuCommand('app.settings', () => navigateTo('/settings'))
  useMenuCommand('app.account', () => navigateTo({ path: '/settings', query: { tab: 'cloud' } }))
  useMenuCommand('edit.undo', () => historyStep(false))
  useMenuCommand('edit.redo', () => historyStep(true))
  useMenuCommand('file.showPluginsFolder', () => plugins.showFolder())

  const context = computed<MenuContext>(() => ({
    undo: typing.value || history.canUndo.value || history.hasPending.value,
    redo: typing.value || history.canRedo.value,
    editor: route.path === '/',
    key: selection.value !== null,
  }))
  watch(context, (value) => {
    commands.setMenuContext(value).catch(() => {})
  }, { immediate: true })

  if (!listening) {
    listening = true
    events.menuCommand.listen(({ payload }) => dispatch(payload.command))
  }
}
