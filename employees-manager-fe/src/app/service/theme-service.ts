import { Injectable } from '@angular/core';

const STORAGE_KEY = 'storage-key-theme';

@Injectable({
  providedIn: 'root',
})
export class ThemeService {
  darkTheme: boolean = false;

  constructor() {
    try {
      const storageValue = localStorage.getItem(STORAGE_KEY);
      this.darkTheme = storageValue !== null ? storageValue === 'true' : false;
    } catch (error) {
      this.darkTheme = false;
    }
    if (this.darkTheme) {
      document.body.classList.toggle('dark-mode');
    }
  }

  toggleDarkTheme() {
    this.darkTheme = !this.darkTheme;
    document.body.classList.toggle('dark-mode');
    localStorage.setItem(STORAGE_KEY, `${this.darkTheme}`);
  }

  clear() {
    this.darkTheme = false;
    localStorage.removeItem(STORAGE_KEY);
  }

  isDarkTheme(): boolean {
    return this.darkTheme;
  }
}
