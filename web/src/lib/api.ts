import axios, { type AxiosResponse } from "axios";
import type { ApiAuthzRes, ApiGetArticlesRes, Session } from "./types";

export const authz = async (
  stateParam: Option<string>
): Promise<AxiosResponse<ApiAuthzRes>> => {
  return axios.post(
    `${import.meta.env.VITE_PUBLIC_APP_SERVER_BASE_URL}/auth/authz`,
    {
      state: stateParam || null,
    },
    {
      withCredentials: true,
    }
  );
}

export const getSession = async (): Promise<AxiosResponse<Session>> =>
  axios.get(`${import.meta.env.VITE_PUBLIC_APP_SERVER_BASE_URL}/auth/session`,
    {
      withCredentials: true,
    }
  );

export const PER_PAGE = 30;
export const getArticles = async (page: number = 0): Promise<
  AxiosResponse<ApiGetArticlesRes>
> =>
  axios.get(`${import.meta.env.VITE_PUBLIC_APP_SERVER_BASE_URL}/articles?page=${page}`,
    {
      withCredentials: true,
    }
  );