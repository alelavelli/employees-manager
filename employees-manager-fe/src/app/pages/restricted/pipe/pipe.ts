import { CommonModule } from '@angular/common';
import { Component, OnInit, ViewEncapsulation } from '@angular/core';
import { EllipsisPipe } from '../../../pipes/ellipsis.pipe';

@Component({
  selector: 'pipe-page',
  templateUrl: './pipe.html',
  styleUrls: ['./pipe.scss'],
  standalone: true,
  imports: [CommonModule, EllipsisPipe],
  encapsulation: ViewEncapsulation.None,
})
export class PipePageComponent implements OnInit {
  constructor() {}

  ngOnInit(): void {}
}
