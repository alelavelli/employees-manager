import { AsyncPipe, CommonModule } from '@angular/common';
import {
  ChangeDetectionStrategy,
  Component,
  computed,
  EventEmitter,
  Input,
  model,
  OnInit,
  Output,
  signal,
  SimpleChanges,
  ViewEncapsulation,
} from '@angular/core';
import { FormsModule, ReactiveFormsModule } from '@angular/forms';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';
import { MatSelectModule } from '@angular/material/select';
import {
  MatAutocompleteModule,
  MatAutocompleteSelectedEvent,
} from '@angular/material/autocomplete';
import { MatChipInputEvent, MatChipsModule } from '@angular/material/chips';
import { MatIconModule } from '@angular/material/icon';
import { COMMA, ENTER } from '@angular/cdk/keycodes';
import { EllipsisPipe } from '../../pipes/ellipsis.pipe';
import { IdLabelElement } from '../../types/model';

@Component({
  selector: 'input-auto-complete',
  templateUrl: './input-auto-complete.html',
  styleUrls: ['./input-auto-complete.scss'],
  standalone: true,
  imports: [
    CommonModule,
    MatInputModule,
    MatFormFieldModule,
    MatSelectModule,
    MatAutocompleteModule,
    ReactiveFormsModule,
    MatChipsModule,
    MatIconModule,
    FormsModule,
    EllipsisPipe,
  ],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class InputAutoCompleteComponent implements OnInit {
  @Input() allElements: IdLabelElement[] = [];
  @Input() elementName: string = '';
  @Input() resetSignal: boolean = false;

  @Output() onAdd: EventEmitter<IdLabelElement> = new EventEmitter();
  @Output() onRemove: EventEmitter<IdLabelElement> = new EventEmitter();

  separatorKeysCodes: number[] = [ENTER, COMMA];

  // the element that is currently typed in the input component
  currentElement = model('');
  // elements already selected from the list allElements
  selectedElements = signal([] as string[]);
  // elements from allElements filtered by currentElement input that
  // are not already selected
  filteredElements = computed(() => {
    const currentElement = this.currentElement().toLocaleLowerCase();
    const selectedElements = new Set(this.selectedElements());
    return currentElement
      ? this.allElements
          .filter((element) =>
            element.label.toLocaleLowerCase().includes(currentElement)
          )
          .filter((element) => !selectedElements.has(element.label))
      : this.allElements
          .slice()
          .filter((element) => !selectedElements.has(element.label));
  });

  ngOnInit(): void {
    console.log('allElements', this.allElements);
  }

  ngOnChanges(changes: SimpleChanges) {
    if (changes['resetSignal']) {
      this.selectedElements.set([]);
      this.currentElement.set('');
    }
  }

  getElementObject(elementLabel: string): IdLabelElement | undefined {
    return this.allElements.filter(
      (element) => element.label === elementLabel
    )[0];
  }

  removeElement(element: string): void {
    this.selectedElements.update((elements) => {
      const index = elements.indexOf(element);
      if (index < 0) {
        return elements;
      } else {
        const deleted_element = this.getElementObject(
          elements.splice(index, 1)[0]
        )!;
        this.onRemove.emit(deleted_element);
        return [...elements];
      }
    });
  }

  addElement(event: MatChipInputEvent): void {
    const value = (event.value || '').trim();
    this.currentElement.set('');
    if (value && this.allElements.map((el) => el.label).includes(value)) {
      this.selectedElements.update((elements) => [...elements, value]);
      const added_element = this.getElementObject(value)!;
      this.onAdd.emit(added_element);
    }
  }

  selectedElement(event: MatAutocompleteSelectedEvent): void {
    const value = event.option.viewValue;
    this.selectedElements.update((elements) => [...elements, value]);
    const added_element = this.getElementObject(value)!;
    this.onAdd.emit(added_element);
    this.currentElement.set('');
    event.option.deselect();
  }
}
