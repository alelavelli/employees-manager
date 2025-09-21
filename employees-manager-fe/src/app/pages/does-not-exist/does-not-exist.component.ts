import { Component, OnDestroy, OnInit } from '@angular/core';
import { Router } from '@angular/router';
import { UserService } from '../../service/user.service';

@Component({
  selector: 'unauthorized',
  templateUrl: './does-not-exist.component.html',
  styleUrls: ['./does-not-exist.component.scss'],
  standalone: true,
})
export class DoesNotExistPageComponent implements OnInit, OnDestroy {
  constructor() {}

  ngOnInit(): void {}
  ngOnDestroy(): void {}
}
