import { CommonModule } from '@angular/common';
import { Component, OnInit, ViewEncapsulation } from '@angular/core';
import { MatButtonModule } from '@angular/material/button';
import { MatIconModule } from '@angular/material/icon';
import { MatMenuModule } from '@angular/material/menu';
import { Router, RouterModule } from '@angular/router';
import { UserService } from '../../service/user.service';

@Component({
  selector: 'user-widget',
  standalone: true,
  imports: [
    CommonModule,
    MatMenuModule,
    MatIconModule,
    MatButtonModule,
    RouterModule,
  ],
  templateUrl: './user-widget.html',
  styleUrl: './user-widget.scss',
  encapsulation: ViewEncapsulation.None,
})
export class UserWidgetComponent implements OnInit {
  constructor(private userService: UserService, private router: Router) {}

  ngOnInit(): void {}

  logout() {
    this.userService.clear();
    this.router.navigateByUrl('/login');
  }
}
