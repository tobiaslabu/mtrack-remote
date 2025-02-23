# mtrack-remote

mtrack-remote is a remote web application for use with [mtrack](https://github.com/mdwn/mtrack).

## Running mtrack-remote

mtrack-remote is built with [Dioxus](https://dioxuslabs.com/) and uses [TailwindCSS](https://tailwindcss.com/). 

In order to run mtrack-remote locally, you can use Dioxus's tooling: 

    dx serve --platform web

To rebuild the stylesheets use Tailwind CLI:

    npx @tailwindcss/cli -i styling/input.css -o assets/tailwind.css

