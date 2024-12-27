import { Component, OnDestroy, OnInit } from '@angular/core';
import { Router } from '@angular/router';
import { UserService } from '../../service/user.service';

@Component({
  selector: 'unauthorized',
  templateUrl: './unauthorized.component.html',
  styleUrls: ['./unauthorized.component.scss'],
  standalone: true,
})
export class UnauthorizedPageComponent implements OnInit, OnDestroy {
  constructor(private userService: UserService, private router: Router) {}

  ngOnInit(): void {
    setTimeout(() => {
      if (this.userService.isAuthenticated()) {
        this.router.navigateByUrl('/home');
      } else {
        this.router.navigateByUrl('/login');
      }
    }, 3000);
  }
  ngOnDestroy(): void {}
}
