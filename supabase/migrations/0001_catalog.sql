-- OpenExamTrainer — catalog, ratings, challenges and progress sync.
--
-- NOT YET APPLIED. This file has never run against a database: the project it needs does not
-- exist. Treat every policy below as a proposal to review, not as deployed behaviour.
--
-- The desktop app works without any of this. Nothing here is a precondition for importing,
-- drilling, reviewing, exporting or recording — it only adds sharing and cross-machine sync.

-- ---------------------------------------------------------------------------------------------
-- Published binders
-- ---------------------------------------------------------------------------------------------

create table public.published_binders (
    id              uuid primary key default gen_random_uuid(),
    owner_id        uuid not null references auth.users (id) on delete cascade,
    title           text not null check (length(title) between 1 and 200),
    certification   text not null check (length(certification) between 1 and 40),
    -- Denormalised from the deck so the catalog can sort and filter without opening the zip.
    question_count  int  not null check (question_count >= 0),
    needs_source_count int not null default 0 check (needs_source_count >= 0),
    profile         text not null,
    -- Path in the `decks` storage bucket. The .examdeck itself is never stored in a column.
    storage_path    text not null unique,
    published_at    timestamptz not null default now(),
    updated_at      timestamptz not null default now(),
    rating_count    int  not null default 0,
    rating_sum      int  not null default 0
);

create index published_binders_certification on public.published_binders (certification);
create index published_binders_published_at   on public.published_binders (published_at desc);

alter table public.published_binders enable row level security;

-- A published binder is world-readable by design: the catalog is the point.
create policy "anyone may read the catalog"
    on public.published_binders for select
    using (true);

create policy "an author may publish"
    on public.published_binders for insert
    to authenticated
    with check (auth.uid() = owner_id);

-- `rating_count` and `rating_sum` are maintained by the trigger below, never by the owner:
-- an author who could write them could invent their own score.
create policy "an author may update their own binder"
    on public.published_binders for update
    to authenticated
    using (auth.uid() = owner_id)
    with check (auth.uid() = owner_id);

create policy "an author may withdraw their own binder"
    on public.published_binders for delete
    to authenticated
    using (auth.uid() = owner_id);

-- ---------------------------------------------------------------------------------------------
-- Ratings
-- ---------------------------------------------------------------------------------------------

create table public.ratings (
    binder_id  uuid not null references public.published_binders (id) on delete cascade,
    rater_id   uuid not null references auth.users (id) on delete cascade,
    stars      int  not null check (stars between 1 and 5),
    comment    text check (length(comment) <= 2000),
    rated_at   timestamptz not null default now(),
    -- One rating per person per binder. Without this, a rating is a vote counter.
    primary key (binder_id, rater_id)
);

alter table public.ratings enable row level security;

create policy "anyone may read ratings"
    on public.ratings for select
    using (true);

create policy "a signed-in user may rate once"
    on public.ratings for insert
    to authenticated
    with check (auth.uid() = rater_id);

create policy "a rater may change their own rating"
    on public.ratings for update
    to authenticated
    using (auth.uid() = rater_id)
    with check (auth.uid() = rater_id);

create policy "a rater may withdraw their own rating"
    on public.ratings for delete
    to authenticated
    using (auth.uid() = rater_id);

-- Keeps the denormalised aggregate honest without granting anyone write access to it.
create or replace function public.apply_rating() returns trigger
language plpgsql security definer set search_path = public as $$
begin
    if tg_op = 'INSERT' then
        update public.published_binders
           set rating_count = rating_count + 1,
               rating_sum   = rating_sum + new.stars
         where id = new.binder_id;
    elsif tg_op = 'UPDATE' then
        update public.published_binders
           set rating_sum = rating_sum - old.stars + new.stars
         where id = new.binder_id;
    else
        update public.published_binders
           set rating_count = rating_count - 1,
               rating_sum   = rating_sum - old.stars
         where id = old.binder_id;
    end if;
    return null;
end;
$$;

create trigger ratings_maintain_aggregate
    after insert or update or delete on public.ratings
    for each row execute function public.apply_rating();

-- ---------------------------------------------------------------------------------------------
-- Challenge results
-- ---------------------------------------------------------------------------------------------

create table public.challenge_results (
    id             uuid primary key default gen_random_uuid(),
    binder_id      uuid not null references public.published_binders (id) on delete cascade,
    runner_id      uuid not null references auth.users (id) on delete cascade,
    -- The seed is what makes two runs comparable; see db.rs `session_questions`.
    seed           bigint not null,
    question_count int  not null check (question_count > 0),
    correct        int  not null check (correct >= 0),
    elapsed_ms     bigint not null check (elapsed_ms >= 0),
    finished_at    timestamptz not null default now(),
    constraint correct_within_bounds check (correct <= question_count)
);

create index challenge_results_board
    on public.challenge_results (binder_id, seed, correct desc, elapsed_ms asc);

alter table public.challenge_results enable row level security;

create policy "anyone may read a leaderboard"
    on public.challenge_results for select
    using (true);

create policy "a runner may post their own result"
    on public.challenge_results for insert
    to authenticated
    with check (auth.uid() = runner_id);

-- Deliberately no update policy: a posted time is a record, and an editable record is not one.

-- ---------------------------------------------------------------------------------------------
-- Progress sync
-- ---------------------------------------------------------------------------------------------

-- Mirrors the local `attempts` and `scheduling` tables, keyed by the *content* of the question
-- rather than its local row id — ids are per-machine, so syncing them would pair the wrong rows.
--
-- The whole FSRS card is here rather than stability and a due date alone: the columns left out of
-- an earlier draft of this file are read by the scheduler, so the machine receiving a partial card
-- would have to invent them. `src-tauri/src/catalog.rs` holds the same table in SQLite and is the
-- one that runs today; the two are kept column for column.
create table public.progress (
    user_id        uuid not null references auth.users (id) on delete cascade,
    question_key   text not null,
    attempts       int  not null default 0 check (attempts >= 0),
    correct        int  not null default 0 check (correct >= 0),
    due_at         timestamptz,
    last_review_at timestamptz,
    stability      real not null default 0,
    difficulty     real not null default 0,
    elapsed_days   int  not null default 0,
    scheduled_days int  not null default 0,
    reps           int  not null default 0,
    lapses         int  not null default 0 check (lapses >= 0),
    state          int  not null default 0,
    updated_at     timestamptz not null default now(),
    primary key (user_id, question_key),
    constraint correct_within_attempts check (correct <= attempts)
);

alter table public.progress enable row level security;

-- Progress is private. There is no read policy for anyone but the owner, on purpose.
create policy "a user may read their own progress"
    on public.progress for select
    to authenticated
    using (auth.uid() = user_id);

create policy "a user may write their own progress"
    on public.progress for insert
    to authenticated
    with check (auth.uid() = user_id);

create policy "a user may update their own progress"
    on public.progress for update
    to authenticated
    using (auth.uid() = user_id)
    with check (auth.uid() = user_id);
