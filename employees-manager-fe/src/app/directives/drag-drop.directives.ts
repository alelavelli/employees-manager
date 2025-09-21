import {
  Directive,
  HostBinding,
  HostListener,
  Output,
  EventEmitter,
} from '@angular/core';
import { DomSanitizer, SafeUrl } from '@angular/platform-browser';

const NORMAL_BACKGROUND = '#fde6e8';
const DRAG_BACKGROUND = 'pink';

@Directive({
  selector: '[fileDrop]',
})
export class DragDirective {
  @Output() onFileDropped: EventEmitter<File> = new EventEmitter();

  @HostBinding('style.background-color') private background = NORMAL_BACKGROUND;

  constructor(private sanitizer: DomSanitizer) {}

  @HostListener('dragover', ['$event']) public onDragOver(evt: DragEvent) {
    evt.preventDefault();
    evt.stopPropagation();
    this.background = DRAG_BACKGROUND;
  }

  @HostListener('dragleave', ['$event']) public onDragLeave(evt: DragEvent) {
    evt.preventDefault();
    evt.stopPropagation();
    this.background = NORMAL_BACKGROUND;
  }

  @HostListener('drop', ['$event']) public onDrop(evt: DragEvent) {
    evt.preventDefault();
    evt.stopPropagation();
    this.background = NORMAL_BACKGROUND;
  }
}
