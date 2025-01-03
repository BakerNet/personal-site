/** @type {import('tailwindcss').Config} */
module.exports = {
  content: {
    relative: true,
    files: ["*.html", "./src/**/*.rs"],
  },
  theme: {
    colors: {
      "background": "#2e3440",
      "foreground": "#cdcecf",
      "cursorColor": "#cdcecf",
      "selectionBackground": "#3e4a5b",
      "black": "#3b4252",
      "blue": "#81a1c1",
      "cyan": "#88c0d0",
      "green": "#a3be8c",
      "purple": "#b48ead",
      "red": "#bf616a",
      "white": "#e5e9f0",
      "yellow": "#ebcb8b",
      "orange": "#d08770",
      "brightBlack": "#465780",
      "brightBlue": "#8cafd2",
      "brightCyan": "#93ccdc",
      "brightGreen": "#b1d196",
      "brightPurple": "#c895bf",
      "brightRed": "#d06f79",
      "brightWhite": "#e7ecf4",
      "brightYellow": "#f0d399",
    },
    extend: {
      keyframes: {
        blink: {
          '50%': { opacity: 0 },
        },
      },
      animation: {
        blink: 'blink 1s ease-in-out infinite',
      },
    },
  },
  plugins: [],
}

