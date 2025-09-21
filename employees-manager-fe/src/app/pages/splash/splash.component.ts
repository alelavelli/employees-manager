import { Component, ViewEncapsulation, OnInit } from '@angular/core';
import { Router } from '@angular/router';
import { UserService } from '../../service/user.service';

@Component({
  selector: 'splash-page',
  templateUrl: './splash.component.html',
  styleUrls: ['./splash.component.scss'],
  encapsulation: ViewEncapsulation.None,
})
export class SplashPageComponent implements OnInit {
  constructor(private userService: UserService, private router: Router) {}

  ngOnInit(): void {}

  onAnimationEnd(): void {
    this.router.navigateByUrl(
      this.userService.isAuthenticated() ? '/home' : '/login'
    );
  }
}
