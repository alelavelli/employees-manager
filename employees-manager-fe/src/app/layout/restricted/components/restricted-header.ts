import { Component, OnInit, ViewEncapsulation } from '@angular/core';
import { UserWidgetComponent } from '../../../components/user-widget/user-widget';
import { RouterModule } from '@angular/router';
import { DarkModeWidgetComponent } from '../../../components/dark-mode-widget/dark-mode-widget';

@Component({
  selector: 'restricted-header',
  templateUrl: './restricted-header.html',
  styleUrls: ['./restricted-header.scss'],
  encapsulation: ViewEncapsulation.None,
  standalone: true,
  imports: [UserWidgetComponent, DarkModeWidgetComponent, RouterModule],
})
export class RestrictedHeaderComponent implements OnInit {
  constructor() {}

  ngOnInit(): void {}
}
