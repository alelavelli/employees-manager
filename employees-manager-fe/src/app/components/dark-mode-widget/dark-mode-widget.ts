import { Component, OnInit, ViewEncapsulation } from '@angular/core';
import { ThemeService } from '../../service/theme-service';
import { CommonModule } from '@angular/common';
import { MatIcon } from '@angular/material/icon';

@Component({
  selector: 'dark-mode-widget',
  standalone: true,
  imports: [CommonModule, MatIcon],
  templateUrl: './dark-mode-widget.html',
  styleUrl: './dark-mode-widget.scss',
  encapsulation: ViewEncapsulation.None,
})
export class DarkModeWidgetComponent implements OnInit {
  darkTheme: boolean = false;

  constructor(private themeService: ThemeService) {}

  ngOnInit(): void {
    this.darkTheme = this.themeService.darkTheme;
  }

  toggle() {
    this.themeService.toggleDarkTheme();
    this.darkTheme = this.themeService.darkTheme;
  }
}
