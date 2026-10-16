export type ToastTone = 'success' | 'error' | 'info'
export interface ToastMessage {
  id: number
  message: string
  tone: ToastTone
}

export function useToast() {
  const messages = useState<ToastMessage[]>('action-toasts', () => [])
  const sequence = useState('action-toast-sequence', () => 0)
  function show(message: string, tone: ToastTone = 'success') {
    if (!import.meta.client || !message) return
    const id = ++sequence.value
    messages.value = [...messages.value.slice(-4), { id, message, tone }]
  }
  function dismiss(id: number) {
    messages.value = messages.value.filter((message) => message.id !== id)
  }
  return { messages, show, dismiss }
}

// Clear transient feedback after delivery so identical repeated actions are announced.
export function useActionFeedback(source: Ref<string>, tone: ToastTone = 'error') {
  const toast = useToast()
  watch(
    source,
    (message) => {
      if (!message) return
      toast.show(message, tone)
      source.value = ''
    },
    { flush: 'sync' },
  )
}
