--#DATABASE=http_stream_select
--#SORT=true
--#CHUNKED=true
DROP DATABASE IF EXISTS http_stream_select;
CREATE DATABASE http_stream_select WITH TTL '100000d';
--#LP_BEGIN
m0,t0=t0 f0=false,f1=0.0 0
--#LP_END
INSERT m0(TIME, f0, f1) VALUES(366012624080382889, FALSE, 0.6326195071473769), (5635692422062413216, (((CAST(0.7382978061623816 AS STRING))||(CAST(FALSE AS STRING)))) NOT IN ('9sP䋛zqeoM', 'k*G'), 0.5469987105166848), (7533102572643168002, TRUE, 0.9622249970170084); -- 0ms;
INSERT m0(TIME, f0, f1) VALUES(4283635885038645395, TRUE, 0.4131558313302025), (6292339431914973050, TRUE, 0.5540439337463335); -- 1ms;
INSERT m0(TIME, f1, t0) VALUES(1041670293467254361, 0.507623643211476, '916053861'), (3174128646074400477, 0.47166914414715877, NULL); -- 0ms;
-- INSERT m0(TIME, f0) VALUES(2079939785551584142, NULL), (1243152233754651379, FALSE); -- 0ms;
INSERT m0(TIME, f0) VALUES(631407052613557553, TRUE), (7486831592909450783, TRUE); -- 0ms;
INSERT m0(TIME, f0) VALUES(5867172425191822176, TRUE), (3986678807649375642, TRUE); -- 0ms;
SELECT m0.t0 FROM m0 WHERE NOT (((NOT (m0.f0)) IS UNKNOWN) BETWEEN SYMMETRIC (CAST((m0.f0) IS UNKNOWN AS BOOLEAN)) AND (CAST(CAST(415483039 AS BIGINT) AS BOOLEAN))) UNION ALL SELECT m0.t0 FROM m0 WHERE NOT (NOT (((NOT (m0.f0)) IS UNKNOWN) BETWEEN SYMMETRIC (CAST((m0.f0) IS UNKNOWN AS BOOLEAN)) AND (CAST(CAST(415483039 AS BIGINT) AS BOOLEAN)))) UNION ALL SELECT m0.t0 FROM m0 WHERE (NOT (((NOT (m0.f0)) IS UNKNOWN) BETWEEN SYMMETRIC (CAST((m0.f0) IS UNKNOWN AS BOOLEAN)) AND (CAST(CAST(415483039 AS BIGINT) AS BOOLEAN)))) IS NULL;
SELECT m0.t0 FROM m0 WHERE NOT (((NOT (m0.f0)) IS UNKNOWN) BETWEEN SYMMETRIC (CAST((m0.f0) IS UNKNOWN AS BOOLEAN)) AND (CAST(CAST(415483039 AS BIGINT) AS BOOLEAN))) UNION ALL SELECT m0.t0 FROM m0 WHERE NOT (NOT (((NOT (m0.f0)) IS UNKNOWN) BETWEEN SYMMETRIC (CAST((m0.f0) IS UNKNOWN AS BOOLEAN)) AND (CAST(CAST(415483039 AS BIGINT) AS BOOLEAN)))) UNION ALL SELECT m0.t0 FROM m0 WHERE (NOT (((NOT (m0.f0)) IS UNKNOWN) BETWEEN SYMMETRIC (CAST((m0.f0) IS UNKNOWN AS BOOLEAN)) AND (CAST(CAST(415483039 AS BIGINT) AS BOOLEAN)))) IS NULL;
SELECT m0.f0, m0.t0, m0.f1 FROM m0 order by time asc;
-- INSERT m0(TIME, t0) VALUES(5414775681413349294, '');
SELECT * FROM m0 order by time asc;
