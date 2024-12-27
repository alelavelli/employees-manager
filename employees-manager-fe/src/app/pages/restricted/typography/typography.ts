import { Component, OnInit, ViewEncapsulation } from '@angular/core';

@Component({
  selector: 'typography-page',
  templateUrl: './typography.html',
  styleUrls: ['./typography.scss'],
  standalone: true,
  encapsulation: ViewEncapsulation.None,
})
export class TypographyPageComponent implements OnInit {
  constructor() {}

  ngOnInit(): void {}
}
