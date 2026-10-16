import { invoke } from '@tauri-apps/api/core';
import type { DecisionResult, HistoryPage, HistoryQuery, ReviewDecision, ReviewItem, ReviewPage, ReviewQuery } from '../types/review';
export const reviewList = (query: ReviewQuery) => invoke<ReviewPage>('review_list', {query});
export const reviewGet = (id: string) => invoke<ReviewItem>('review_get', {id});
export const reviewDecide = (input: ReviewDecision) => invoke<DecisionResult[]>('review_decide', {input});
export const reviewHistory = (query: HistoryQuery) => invoke<HistoryPage>('review_history', {query});
