// Copyright 2021 Solly Ross

class Slide extends HTMLElement {
    constructor() {
        super();

        const root = this.attachShadow({mode: 'open'});

        // force a master update
        this._setMasterSlide(this.masterSlide);
    }

    set masterSlide(val) {
        this.setAttribute("master", newVal)
    }

    get masterSlide() {
        return this.getAttribute("master") || "default";
    }

    _setMasterSlide(val) {
        if (!val) { val = "default" }
        this.shadowRoot.innerHTML = "";
        this.shadowRoot.appendChild(document.getElementById(`master-${val}`).content.cloneNode(true));
    }

    attributeChangedCallback(attr, oldVal, newVal) {
        switch (attr) {
        case "master":
            this._setMasterSlide(newVal);
            break;
        }
    }

    static get observedAttributes() {
        return ['master'];
    }
}

window.customElements.define('pres-slide', Slide);

const $ = document.querySelector.bind(document);
document.addEventListener("keydown", (evt) => {
    const mainElem = document.querySelector('main');
    switch (evt.code) {
    case 'Space':
    case 'ArrowRight':
        evt.preventDefault();
        return false;
        break;
    case 'ArrowLeft':
        evt.preventDefault();
        return false;
        break
    }
});
document.addEventListener("keyup", (evt) => {
    const mainElem = document.querySelector('main');
    const targetElem = mainElem.querySelector(':target');

    let currentSlide;
    if (targetElem) {
        currentSlide = targetElem.closest('pres-slide');
    }

    if (!currentSlide) {
        // find the first slide that starts in the viewport
        for (let slide of mainElem.querySelectorAll('main > pres-slide')) {
            let bounds = slide.getBoundingClientRect()
            if (bounds.left >= 0 && bounds.top >= 0) {
                currentSlide = slide;
                break;
            }
        }
    }

    const currentInd = Number.parseInt(currentSlide.id.slice("slide-".length));
    let nextInd = currentInd;

    switch (evt.code) {
    case 'Space':
    case 'ArrowRight':
        evt.preventDefault();
        nextInd++;
        break;
    case 'ArrowLeft':
        evt.preventDefault();
        nextInd--;
        break
    }

    if (nextInd < 0 || !$(`#slide-${nextInd}`)) {
        return;
    }

    let next_url = new URL(document.location);
    next_url.hash = `#slide-${nextInd}`;
    window.location.replace(next_url);  // replace so we don't bloat up the history
});
