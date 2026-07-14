import { Component } from '@angular/core'

@Component({
  selector: 'app-header',
  standalone: true,
  template: `<header class="border-b border-gray-200 bg-white py-4">
    <div class="mx-auto max-w-7xl px-4">
      <a routerLink="/" class="text-xl font-bold">{{ name }}</a>
    </div>
  </header>`,
  styles: [``],
})
export class HeaderComponent {}
