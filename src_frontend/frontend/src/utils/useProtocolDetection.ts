import { ref } from 'vue';

export function useProtocolDetection() {
  const { protocol, hostname, port } = new URL(window.location.href);
  const isHttpsProtocol = ref(protocol === 'https:' || hostname === 'localhost');
  const isDevelopLocalhost = ref(hostname === 'localhost' && port === '4080');
  return { isHttpsProtocol, isDevelopLocalhost };
}
