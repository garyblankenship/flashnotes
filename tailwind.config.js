/** @type {import('tailwindcss').Config} */
export default {
  content: ['./src/**/*.{html,js,svelte,ts}'],
  darkMode: 'class',
  theme: {
    extend: {
      fontFamily: {
        sans: ['JetBrains Mono', '-apple-system', 'BlinkMacSystemFont', 'SF Pro Text', 'sans-serif'],
        mono: ['JetBrains Mono', 'SF Mono', 'Menlo', 'monospace'],
      },
      colors: {
        'zed': {
          'app': '#151515',
          'editor': '#191919',
          'sidebar': '#151515',
          'active': '#252525',
          'border': '#333333',
          'text': '#EBEBEC',
          'muted': '#868686',
          'accent': '#5898F8',
        }
      }
    },
  },
  plugins: [],
};
