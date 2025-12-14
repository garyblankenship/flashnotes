/**
 * Toast notification store
 * Provides a simple, reusable toast notification system
 */

class ToastStore {
  message = $state<string | null>(null);
  private timeout: ReturnType<typeof setTimeout> | null = null;

  /**
   * Show a toast message for a duration
   * @param message - Message to display
   * @param duration - Duration in ms (default 2000)
   */
  show(message: string, duration = 2000) {
    if (this.timeout) {
      clearTimeout(this.timeout);
    }
    this.message = message;
    this.timeout = setTimeout(() => {
      this.message = null;
      this.timeout = null;
    }, duration);
  }

  /**
   * Immediately dismiss the current toast
   */
  dismiss() {
    if (this.timeout) {
      clearTimeout(this.timeout);
      this.timeout = null;
    }
    this.message = null;
  }
}

export const toastStore = new ToastStore();
