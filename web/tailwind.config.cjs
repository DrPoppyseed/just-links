const defaultTheme = require("tailwindcss/defaultTheme");

/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./src/**/*.{html,js,svelte,ts}"],
  theme: {
    extend: {
      fontFamily: {
        serif: ["Cormorant Garamond", ...defaultTheme.fontFamily.serif],
      },
    },
  },
  plugins: [],
};
