import { ref } from 'vue';

export function useMessageModal() {
  const isMessageModal = ref(false);
  const messageText = ref('');
  const messageModalOpenClose = (message: string): void => {
    if (!isMessageModal.value) {
      messageText.value = message;
      isMessageModal.value = true;
    } else {
      isMessageModal.value = false;
      messageText.value = '';
    }
  };
  return { isMessageModal, messageText, messageModalOpenClose };
}
