---
type: SEIObservation
title: Ensaio live da busca do destino de processo enviar no GO 5.0.3
description: Uma prévia somente leitura de ies processo enviar em Goiás 5.0.3 para um destino com barra em outro órgão (ABC/GECONT-18141), 04/11/2026; as três consultas planejadas saíram com os corpos esperados e a recusa tipada veio da última; o destino não foi achado
---

# Busca do destino no GO 5.0.3 — 04/11/2026

Contrato em [processo enviar](../../contracts/cli/processo-enviar.md);
regra da busca na [decisão](../decisoes/processo-enviar.md). Binário de
depuração do ramo `slice-enviar-go` (commit `540dd9f`), sessão de GO
aberta às 21:21 UTC na [varredura](2026-11-04-varredura-unb-go.md) (conta
pública `aluno05`, unidade `SEAD/CDE-09516`), arquivos `0600`. Pedidos
contados por um relay `CONNECT` local (um túnel por pedido). **Uma**
execução, 47 min depois do comando anterior em GO; nenhum retry, nenhum
relogin (a sessão valia). **Nenhum POST de escrita, nenhum `--confirmar`.**

## Escolha

- Processo `202400013000003`: aberto em `SEAD/CDE-09516` (a unidade da
  sessão), `CASACIVIL/COMSET-12318` e `SEAD/-18214`. Já fora aberto na
  varredura (21:31): o GET da tela do processo não registrou recebimento
  novo (abrir pode registrar, aceitável em treinamento).
- Destino `ABC/GECONT-18141`: sigla com `/` em outro órgão (`ABC`, opção
  `28` do `selOrgao`), o caso que o Todos mediu.

## Resultado

| Hora (UTC) | Comando | Saída | Túneis |
| --- | --- | --- | --- |
| 22:21:02 | `processo enviar 202400013000003 --para ABC/GECONT-18141 --json` | 1, `destino_nao_oferecido_pelo_sei` | 6 |

| Túnel | Pedido | Enviado | Recebido |
| --- | --- | --- | --- |
| 1 | GET tela do processo (link guardado) | 1.441 B | 49.372 B |
| 2 | GET árvore | 1.456 B | 84.865 B |
| 3 | GET tela "Enviar Processo" | 1.437 B | 33.269 B |
| 4 | POST consulta 1: `ABC/GECONT-18141`, órgão `28` | 1.883 B | 5.834 B |
| 5 | POST consulta 2: `ABC/GECONT-18141`, órgão `""` | 1.881 B | 5.836 B |
| 6 | POST consulta 3: `GECONT`, órgão `""` | 1.869 B | 9.151 B |

O túnel inclui o handshake TLS (~5,8 KB recebidos sem corpo). Os tamanhos
enviados batem com os corpos planejados: a consulta 1 tem 2 bytes a mais
que a 2 (`28`), e a 3 tem 12 a menos (`GECONT` contra
`ABC%2FGECONT-18141`). As consultas 1 e 2 vieram vazias (~5,8 KB, o
handshake mais `<itens></itens>`); a 3 trouxe 39 itens, os `oferecidos`
da saída: as mesmas 39 siglas `…/GECONT-…` de 35 órgãos gravadas na
cassette `grupo-envio-ciclo-go`, sem nenhuma `ABC/…`. Saída sem URL,
cookie ou `infra_hash`.

| Item | Veredito |
| --- | --- |
| Sequência e orçamento da busca com órgão do prefixo (3 GET + 3 POST, abertura quente) | **PASS** |
| Consulta 1 no órgão do prefixo; consulta 3 só depois da 2 vazia | **PASS** (pelos tamanhos) |
| Recusa tipada com `oferecidos` da última resposta | **PASS** |
| Achar `ABC/GECONT-18141` | **FAIL**: nenhuma das três consultas o trouxe |
| Corte em 50 com "Todos" | **NOT_RUN**: nenhuma resposta com 50 itens |
| `Location` `http://` do 302 de escrita | **NOT_RUN**: exige `--confirmar` |

Nenhum 403 nem `bloqueio_de_seguranca_da_instalacao`: 6 pedidos em 14 s.

## Leitura

O filtro pelo órgão com a sigla inteira não bastou no GO: a sigla inteira
não casou nem no órgão `28`. Não se sabe se a unidade é um
destino oferecido; só a consulta que a regra não faz (`GECONT` no órgão
`28`, a quarta do Todos) ou a lupa diriam. Revisão anotada na
[decisão](../decisoes/processo-enviar.md).
