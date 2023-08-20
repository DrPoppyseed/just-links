import axios, { type AxiosResponse } from "axios";
import type { ApiAuthzRes, ApiGetArticlesRes, ApiGetSessionRes } from "./types"

export const authz = async (stateParam: Option<string>): Promise<AxiosResponse<ApiAuthzRes>> => {
  return axios.post(
    `${import.meta.env.VITE_PUBLIC_APP_SERVER_BASE_URL}/auth/authz`,
    {
      state: stateParam || null
    },
    {
      withCredentials: true
    }
  )
}

export const getSession = async (): Promise<AxiosResponse<ApiGetSessionRes>> => {
  return axios.get(
    `${import.meta.env.VITE_PUBLIC_APP_SERVER_BASE_URL}/auth/session`,
    { withCredentials: true }
  )
}

export const getArticles = async (): Promise<AxiosResponse<ApiGetArticlesRes>> => {
  return axios.get(
    `${import.meta.env.VITE_PUBLIC_APP_SERVER_BASE_URL}/articles`,
    { withCredentials: true }
  )
}
