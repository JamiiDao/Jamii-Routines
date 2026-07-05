export default function getHref(route: String) {
    let href = window.location.href

    if (import.meta.env.DEV) {
        href = "http://127.0.0.1:3000/" + route
    }

    return href;
}