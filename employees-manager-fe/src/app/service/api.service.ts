import { HttpClient } from '@angular/common/http';
import { Injectable } from '@angular/core';
import { Observable } from 'rxjs';
import { environment } from '../../environments/environment';
import { buildMocked } from './mock';
import {
  AdminPanelOverview,
  AdminPanelUserInfo,
  AppNotification,
  UserInCompanyInfo,
  CreateUserParameters,
  LoginResponse,
  UserData,
  CompanyInfo,
  CreateCompanyParameters,
  UserToInvite,
  InvitedUserInCompanyInfo,
  CompanyProjectInfo,
} from '../types/model';
import { CompanyRole, NotificationType } from '../types/enums';

const MOCKED = false;
const API_URL = environment.apiHost + '/api';

@Injectable({
  providedIn: 'root',
})
export class ApiService {
  constructor(private httpClient: HttpClient) {}

  login(username: string, password: string): Observable<LoginResponse> {
    return MOCKED
      ? buildMocked({
          token: 'token',
          tokenType: 'Bearer',
        })
      : this.httpClient.post<LoginResponse>(API_URL + '/auth/login', {
          username,
          password,
        });
  }

  getUserData(): Observable<UserData> {
    return MOCKED
      ? buildMocked({
          id: 'my-id',
          username: 'my-username',
          name: 'my-name',
          surname: 'my-surname',
          email: 'my-email',
          platformAdmin: true,
          active: true,
        })
      : this.httpClient.get<UserData>(API_URL + '/auth/user');
  }

  getAdminPanelOverview(): Observable<AdminPanelOverview> {
    return MOCKED
      ? buildMocked({
          totalUsers: 10,
          totalAdmins: 2,
          totalActiveUsers: 8,
          totalInactiveUsers: 2,
          totalCompanies: 5,
        })
      : this.httpClient.get<AdminPanelOverview>(API_URL + '/admin/overview');
  }

  getAdminUsersInfo(): Observable<AdminPanelUserInfo[]> {
    return MOCKED
      ? buildMocked(
          [...Array(50).keys()].map((i) => ({
            id: `my-id-${i}`,
            username: `my-username-${i}`,
            name: `my-name-${i}`,
            surname: `my-surname-${i}`,
            email: `my-email-${i}`,
            platformAdmin: i % 2 === 0,
            active: i % 3 === 0,
            totalCompanies: i,
          }))
        )
      : this.httpClient.get<AdminPanelUserInfo[]>(API_URL + '/admin/user');
  }

  setPlatformAdminUser(userId: string): Observable<void> {
    return MOCKED
      ? buildMocked()
      : this.httpClient.post<void>(
          API_URL + `/admin/user/${userId}/platform-admin`,
          {}
        );
  }

  unsetPlatformAdminUser(userId: string): Observable<void> {
    return MOCKED
      ? buildMocked()
      : this.httpClient.delete<void>(
          API_URL + `/admin/user/${userId}/platform-admin`,
          {}
        );
  }

  activateUser(userId: string): Observable<void> {
    return MOCKED
      ? buildMocked()
      : this.httpClient.post<void>(
          API_URL + `/admin/user/${userId}/activate`,
          {}
        );
  }

  deactivateUser(userId: string): Observable<void> {
    return MOCKED
      ? buildMocked()
      : this.httpClient.delete<void>(
          API_URL + `/admin/user/${userId}/activate`,
          {}
        );
  }

  deleteUser(userId: string): Observable<void> {
    return MOCKED
      ? buildMocked()
      : this.httpClient.delete<void>(API_URL + `/admin/user/${userId}`, {});
  }

  createUser(user: CreateUserParameters): Observable<string> {
    return MOCKED
      ? buildMocked('new-user-id')
      : this.httpClient.post<string>(API_URL + '/admin/user', user);
  }

  getUnreadNotifications(): Observable<AppNotification[]> {
    return MOCKED
      ? buildMocked(
          [...Array(50).keys()].map((i) => ({
            id: `id-${i}`,
            notificationType:
              i % 2 == 0
                ? NotificationType.InviteAddCompany
                : NotificationType.InviteAddCompanyAnswer,
            message:
              i % 2 == 0
                ? `You has been invited to Company ${i}`
                : `The user accepted to join in Company ${i}`,
          }))
        )
      : this.httpClient.get<AppNotification[]>(API_URL + '/notification');
  }

  setNotificationAsRead(notificationId: string): Observable<void> {
    return MOCKED
      ? buildMocked()
      : this.httpClient.patch<void>(
          API_URL + `/notification/${notificationId}/read`,
          {}
        );
  }

  acceptInviteAddCompany(notificationId: string) {
    return MOCKED
      ? buildMocked()
      : this.httpClient.patch<void>(
          API_URL + `/notification/invite-add-company/${notificationId}`,
          {
            accept: true,
          }
        );
  }

  declineInviteAddCompany(notificationId: string) {
    return MOCKED
      ? buildMocked()
      : this.httpClient.patch<void>(
          API_URL + `/notification/invite-add-company/${notificationId}`,
          {
            accept: false,
          }
        );
  }

  getUserCompanies(): Observable<CompanyInfo[]> {
    return MOCKED
      ? buildMocked(
          [...Array(50).keys()].map((i) => ({
            id: `id-${i}`,
            name: `company-${i}`,
            active: i % 3 === 0,
            totalUsers: i * 2,
            role: [CompanyRole.Owner, CompanyRole.Admin, CompanyRole.User][
              i % 3
            ],
          }))
        )
      : this.httpClient.get<CompanyInfo[]>(API_URL + '/company');
  }

  getUsersInCompany(companyId: string): Observable<UserInCompanyInfo[]> {
    return MOCKED
      ? buildMocked(
          [...Array(50).keys()].map((i) => ({
            userId: `user-id-${i}`,
            userName: `name-${i}`,
            userSurname: `surname-${i}`,
            userUsername: `username-${i}`,
            companyId: `company-id-${i}`,
            jobTitle: `job-title-${i}`,
            role: i % 3 === 0 ? CompanyRole.Admin : CompanyRole.User,
            job_title: `job-title-${i}`,
            managementTeam: i % 2 === 0,
          }))
        )
      : this.httpClient.get<UserInCompanyInfo[]>(
          API_URL + `/company/${companyId}/user`
        );
  }

  getPendingUsersInCompany(
    companyId: string
  ): Observable<InvitedUserInCompanyInfo[]> {
    return MOCKED
      ? buildMocked(
          [...Array(50).keys()].map((i) => ({
            userId: `user-id-${i}`,
            notificationId: `notification-id-${i}`,
            username: `name-${i}`,
            companyId: `company-id-${i}`,
            jobTitle: `job-title-${i}`,
            role: i % 3 === 0 ? CompanyRole.Admin : CompanyRole.User,
          }))
        )
      : this.httpClient.get<InvitedUserInCompanyInfo[]>(
          API_URL + `/company/${companyId}/pending-user`
        );
  }

  createCompany(company: CreateCompanyParameters): Observable<string> {
    return MOCKED
      ? buildMocked('new-company-id')
      : this.httpClient.post<string>(API_URL + '/company', company);
  }

  changeUserCompanyRole(
    companyId: string,
    userId: string,
    role: CompanyRole
  ): Observable<void> {
    return MOCKED
      ? buildMocked()
      : this.httpClient.patch<void>(API_URL + `/company/${companyId}/role`, {
          userId: userId,
          role: role,
        });
  }

  changeUserJobTitle(
    companyId: string,
    userId: string,
    jobTitle: string
  ): Observable<void> {
    return MOCKED
      ? buildMocked()
      : this.httpClient.patch<void>(
          API_URL + `/company/${companyId}/job-title`,
          {
            userId: userId,
            jobTitle: jobTitle,
          }
        );
  }

  setUserCompanyManager(companyId: string, userId: string): Observable<void> {
    return MOCKED
      ? buildMocked()
      : this.httpClient.patch<void>(API_URL + `/company/${companyId}/manager`, {
          userId: userId,
          manager: true,
        });
  }

  unsetUserCompanyManager(companyId: string, userId: string): Observable<void> {
    return MOCKED
      ? buildMocked()
      : this.httpClient.patch<void>(API_URL + `/company/${companyId}/manager`, {
          userId: userId,
          manager: false,
        });
  }

  removeUserFromCompany(companyId: string, userId: string): Observable<void> {
    return MOCKED
      ? buildMocked()
      : this.httpClient.delete<void>(
          API_URL + `/company/${companyId}/user/${userId}`,
          {}
        );
  }

  getUsersToInvite(companyId: string): Observable<UserToInvite[]> {
    return MOCKED
      ? buildMocked(
          [...Array(20).keys()].map((i) => ({
            userId: `user-id-${i}`,
            username: `name-${i}`,
          }))
        )
      : this.httpClient.get<UserToInvite[]>(
          API_URL + `/company/${companyId}/user-to-invite`
        );
  }

  inviteUserInCompany(
    companyId: string,
    userId: string,
    role: CompanyRole,
    jobTitle: string,
    projectIds: string[]
  ): Observable<void> {
    return MOCKED
      ? buildMocked()
      : this.httpClient.post<void>(
          API_URL + `/company/${companyId}/invite-user`,
          {
            userId: userId,
            role: role,
            jobTitle: jobTitle,
            projectIds: projectIds,
          }
        );
  }

  cancelInvitation(
    companyId: string,
    notificationId: string
  ): Observable<void> {
    return MOCKED
      ? buildMocked()
      : this.httpClient.delete<void>(
          API_URL + `/company/${companyId}/invite-user/${notificationId}`,
          {}
        );
  }

  getCompanyProjects(companyId: string): Observable<CompanyProjectInfo[]> {
    return MOCKED
      ? buildMocked(
          [...Array(20).keys()].map((i) => ({
            id: `project-${i}`,
            name: `project-name-${i}`,
            code: `code-${i}`,
            active: i % 2 === 0,
          }))
        )
      : this.httpClient.get<CompanyProjectInfo[]>(
          API_URL + `/company/${companyId}/project`
        );
  }

  deleteCompanyProject(companyId: string, projectId: string): Observable<void> {
    return MOCKED
      ? buildMocked()
      : this.httpClient.delete<void>(
          API_URL + `/company/${companyId}/project/${projectId}`
        );
  }

  createCompanyProject(
    companyId: string,
    name: string,
    code: string
  ): Observable<void> {
    return MOCKED
      ? buildMocked()
      : this.httpClient.post<void>(API_URL + `/company/${companyId}/project`, {
          name: name,
          code: code,
        });
  }

  editCompanyProject(
    companyId: string,
    projectId: string,
    name: string,
    code: string,
    active: boolean
  ): Observable<void> {
    return MOCKED
      ? buildMocked()
      : this.httpClient.patch<void>(
          API_URL + `/company/${companyId}/project/${projectId}`,
          {
            name: name,
            code: code,
            active: active,
          }
        );
  }

  getCompanyProjectAllocationsByProject(
    companyId: string,
    projectId: string
  ): Observable<string[]> {
    return MOCKED
      ? buildMocked([...Array(5).keys()].map((i) => `user-id-${i}`))
      : this.httpClient.get<string[]>(
          API_URL + `/company/${companyId}/project-allocation/${projectId}`
        );
  }

  getCompanyProjectAllocationsByUser(
    companyId: string,
    userId: string
  ): Observable<string[]> {
    return MOCKED
      ? buildMocked([...Array(5).keys()].map((i) => `project-${i}`))
      : this.httpClient.get<string[]>(
          API_URL + `/company/${companyId}/user-allocation/${userId}`
        );
  }

  updateCompanyProjectAllocationsByProject(
    companyId: string,
    projectId: string,
    userIds: string[]
  ): Observable<void> {
    return MOCKED
      ? buildMocked()
      : this.httpClient.patch<void>(
          API_URL + `/company/${companyId}/project-allocation/${projectId}`,
          { userIds: userIds }
        );
  }

  updateCompanyProjectAllocationsByUser(
    companyId: string,
    userId: string,
    projectIds: string[]
  ): Observable<void> {
    return MOCKED
      ? buildMocked()
      : this.httpClient.patch<void>(
          API_URL + `/company/${companyId}/user-allocation/${userId}`,
          { projectIds: projectIds }
        );
  }
}
