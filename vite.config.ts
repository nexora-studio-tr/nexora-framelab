import { defineConfig } from 'vitest/config';
export default defineConfig({
  clearScreen: false,
  server: { host: '127.0.0.1', port: 4367, strictPort: true },
  build: {
    target: 'es2022',
    rolldownOptions: {
      onwarn(warning, defaultHandler) {
        if (warning.code === 'MODULE_LEVEL_DIRECTIVE' && warning.message.includes('use client'))
          return;
        defaultHandler(warning);
      },
    },
  },
  test: { include: ['tests/**/*.test.ts'] },
});
