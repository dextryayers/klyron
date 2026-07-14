import { LitElement, css, html } from 'lit'
import { customElement, property } from 'lit/decorators.js'

@customElement('my-element')
export class MyElement extends LitElement {
  @property({ type: Number })
  count = 0

  render() {
    return html`
      <div class="home">
        <h1>{{ name }}</h1>
        <p>{{ description }}</p>
        <div class="card">
          <button @click=${this._onClick}>
            count is ${this.count}
          </button>
        </div>
      </div>
    `
  }

  private _onClick() {
    this.count++
  }

  static styles = css`
    .home {
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      min-height: 80vh;
      text-align: center;
      font-family: system-ui, sans-serif;
    }

    .card {
      padding: 2em;
    }

    button {
      border-radius: 8px;
      border: 1px solid #ccc;
      padding: 0.6em 1.2em;
      font-size: 1em;
      cursor: pointer;
    }

    button:hover {
      border-color: #646cff;
    }

    h1 {
      font-size: 3.2em;
      line-height: 1.1;
    }
  `
}
