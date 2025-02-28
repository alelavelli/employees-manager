import { Component, OnInit, ViewEncapsulation } from '@angular/core';

@Component({
  selector: 'empty-layout',
  templateUrl: './empty-layout.component.html',
  styleUrls: ['./empty-layout.component.scss'],
  encapsulation: ViewEncapsulation.None,
  standalone: true,
})
export class EmptyLayoutComponent implements OnInit {
  constructor() {}

  ngOnInit(): void {}
}
