import { Component, OnInit, ViewEncapsulation } from '@angular/core';
import { RestrictedHeaderComponent } from './components/restricted-header';

@Component({
  selector: 'restricted-layout',
  templateUrl: './restricted-layout.html',
  styleUrls: ['./restricted-layout.scss'],
  encapsulation: ViewEncapsulation.None,
  standalone: true,
  imports: [RestrictedHeaderComponent],
})
export class RestrictedLayoutComponent implements OnInit {
  constructor() {}

  ngOnInit(): void {}
}
