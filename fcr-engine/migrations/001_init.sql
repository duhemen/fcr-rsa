--
-- PostgreSQL database dump
--


-- Dumped from database version 18.1
-- Dumped by pg_dump version 18.1

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET transaction_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', 'public', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: anomaly_events; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.anomaly_events (
    id bigint NOT NULL,
    api_key_id bigint NOT NULL,
    event_type text NOT NULL,
    details text,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    risk_score integer DEFAULT 0,
    distance_km double precision DEFAULT 0.0,
    trace_id character varying(50)
);


--
-- Name: anomaly_events_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.anomaly_events_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: anomaly_events_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.anomaly_events_id_seq OWNED BY public.anomaly_events.id;


--
-- Name: api_key_states; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.api_key_states (
    api_key_id bigint NOT NULL,
    last_timestamp timestamp with time zone,
    updated_at timestamp with time zone DEFAULT now(),
    last_loc_hash character varying(64) DEFAULT ''::character varying NOT NULL
);


--
-- Name: api_keys; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.api_keys (
    id bigint NOT NULL,
    user_name text NOT NULL,
    label text,
    key_prefix text NOT NULL,
    key_hash text NOT NULL,
    is_active boolean DEFAULT true NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    last_used_at timestamp with time zone,
    is_locked boolean DEFAULT false NOT NULL,
    locked_at timestamp with time zone,
    lock_reason text,
    lockout_until timestamp with time zone
);


--
-- Name: api_keys_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.api_keys_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: api_keys_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.api_keys_id_seq OWNED BY public.api_keys.id;


--
-- Name: keys; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.keys (
    id integer NOT NULL,
    name character varying(255) NOT NULL,
    seed text NOT NULL,
    n integer NOT NULL,
    q text NOT NULL,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: keys_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.keys_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: keys_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.keys_id_seq OWNED BY public.keys.id;


--
-- Name: anomaly_events id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.anomaly_events ALTER COLUMN id SET DEFAULT nextval('public.anomaly_events_id_seq'::regclass);


--
-- Name: api_keys id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.api_keys ALTER COLUMN id SET DEFAULT nextval('public.api_keys_id_seq'::regclass);


--
-- Name: keys id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.keys ALTER COLUMN id SET DEFAULT nextval('public.keys_id_seq'::regclass);


--
-- Name: anomaly_events anomaly_events_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.anomaly_events
    ADD CONSTRAINT anomaly_events_pkey PRIMARY KEY (id);


--
-- Name: api_key_states api_key_states_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.api_key_states
    ADD CONSTRAINT api_key_states_pkey PRIMARY KEY (api_key_id);


--
-- Name: api_keys api_keys_key_hash_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.api_keys
    ADD CONSTRAINT api_keys_key_hash_key UNIQUE (key_hash);


--
-- Name: api_keys api_keys_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.api_keys
    ADD CONSTRAINT api_keys_pkey PRIMARY KEY (id);


--
-- Name: keys keys_name_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.keys
    ADD CONSTRAINT keys_name_key UNIQUE (name);


--
-- Name: keys keys_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.keys
    ADD CONSTRAINT keys_pkey PRIMARY KEY (id);


--
-- Name: idx_anomaly_events_key; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_anomaly_events_key ON public.anomaly_events USING btree (api_key_id);


--
-- Name: idx_api_keys_hash_active; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_api_keys_hash_active ON public.api_keys USING btree (key_hash) WHERE is_active;


--
-- Name: anomaly_events anomaly_events_api_key_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.anomaly_events
    ADD CONSTRAINT anomaly_events_api_key_id_fkey FOREIGN KEY (api_key_id) REFERENCES public.api_keys(id);


--
-- Name: api_key_states api_key_states_api_key_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.api_key_states
    ADD CONSTRAINT api_key_states_api_key_id_fkey FOREIGN KEY (api_key_id) REFERENCES public.api_keys(id) ON DELETE CASCADE;


--
-- PostgreSQL database dump complete
--


