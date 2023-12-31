const router = async route => {
    let content = document.getElementById("root");
    content.innerHTML = "";

    let url = route.startsWith("#") ? route.slice(1) : route;

    if (url === "") {
        content.innerHTML = await (await fetch("/home")).text();

        return;
    }

    if (url === "/") {
        url = "/home";
    }

    const response = await fetch(url);

    const body = await response.text();

    content.innerHTML = body;
};

const init = () => {
    router(window.location.hash);

    window.addEventListener("hashchange", () => {
        router(window.location.hash);
    });
};

window.addEventListener("load", init);
