module.exports = {
    mode: "jit",
    content: ["./public/**/*.html", "./src/**/*.{vue,js,ts,jsx,tsx}"],
    purge: ["./public/**/*.html", "./src/**/*.{js,jsx,ts,tsx,vue}"],
    theme: {
        extend: {
            colors: {
                rh: {
                    green: "#045C4C",
                    blue: "#0A2342",
                    white: "#D1DBDA",
                    lightblue: "#7BB2D9",
                    purple: "#3C3D7C",
                },
            },
        },
        fontFamily: {
            archive: ["Archive", "sans-serif"],
            zag: ["Zag", "sans-serif"],
        },
        plugins: [],
    },
};
