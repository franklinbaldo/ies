---
type: SEIObservation
title: Ensaio live da busca do destino de processo enviar no GO 5.0.3
description: Três prévias somente leitura de ies processo enviar em Goiás 5.0.3, 04/11/2026, para destinos com barra em outro órgão; ABC/GECONT-18141, que o Todos registra como inexistente, foi recusado com a lista do termo alargado, e CBM/GECONT-18149 e DETRAN/GECONT-18166 foram achados na primeira consulta, no órgão do prefixo
---

# Busca do destino no GO 5.0.3 — 04/11/2026

Contrato em [processo enviar](../../contracts/cli/processo-enviar.md);
regra da busca na [decisão](../decisoes/processo-enviar.md). Binário de
depuração do ramo `slice-enviar-go` (`540dd9f` na execução 1, até três
consultas; `2c234a1` nas 2 e 3, a ordem do Todos, até quatro), sessão de GO
aberta às 21:21 UTC na [varredura](2026-11-04-varredura-unb-go.md) (conta
pública `aluno05`, unidade `SEAD/CDE-09516`), arquivos `0600`. Pedidos
contados por um relay `CONNECT` local (um túnel por pedido). Três
execuções, a primeira 47 min depois do comando anterior em GO e as outras
com 60 s ou mais entre elas; nenhum retry, nenhum
relogin (a sessão valia). **Nenhum POST de escrita, nenhum `--confirmar`.**

## Escolha

- Processo `202400013000003`: aberto em `SEAD/CDE-09516` (a unidade da
  sessão), `CASACIVIL/COMSET-12318` e `SEAD/-18214`. Já fora aberto na
  varredura (21:31): o GET da tela do processo não registrou recebimento
  novo (abrir pode registrar, aceitável em treinamento).
- Execução 1, `ABC/GECONT-18141`: sigla com `/` em outro órgão (`ABC`,
  opção `28`), escolhida pelos testes do Todos (`test_envio_010.py`). O
  changelog do Todos (`changelog/changes/unidade-pesquisar.md`, medido com
  `unidade_auto_completar_todas`) registra que ela **não existe** e que
  `CBM/GECONT-18149` "acha direto": a escolha do alvo foi errada.
- Execução 2, `CBM/GECONT-18149` (`CBM` = opção `11` do `selOrgao`
  gravado), indicada pelo mesmo changelog.
- Execução 3, `DETRAN/GECONT-18166` (`DETRAN` = opção `25`), da lista de
  39 GECONT da execução 1, procurando uma unidade que só o termo alargado
  achasse dentro do órgão.

## Resultado

| Hora (UTC) | Comando | Saída | Túneis |
| --- | --- | --- | --- |
| 22:21:02 | 1: `processo enviar 202400013000003 --para ABC/GECONT-18141 --json` | 1, `destino_nao_oferecido_pelo_sei` | 6 |
| 22:26:58 | 2: `… --para CBM/GECONT-18149 --json` | 0, `destino` `CBM/GECONT-18149` (id `18149`), `requisicoes: 4` | 4 |
| 22:28:31 | 3: `… --para DETRAN/GECONT-18166 --json` | 0, `destino` `DETRAN/GECONT-18166` (id `18166`), `requisicoes: 4` | 4 |

Execução 1 (versão de até três consultas), túnel a túnel:

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
cassette `grupo-envio-ciclo-go`, sem nenhuma `ABC/…`. Nas execuções 2 e
3, os mesmos três GET e um POST, a consulta 1 da ordem do Todos (sigla
inteira no órgão do prefixo): 1.883 B enviados com `CBM` (o mesmo tamanho
da consulta 1 da execução 1) e 1.886 B com `DETRAN` (3 bytes a mais na
sigla), ~5,9 KB recebidos: um item. Saídas
sem URL, cookie ou `infra_hash`.

| Item | Veredito |
| --- | --- |
| Destino de outro órgão achado no órgão do prefixo, consulta 1 (`CBM/GECONT-18149`, `DETRAN/GECONT-18166`; 3 GET + 1 POST) | **PASS** |
| Unidade inexistente (`ABC/GECONT-18141`) recusada com `destino_nao_oferecido_pelo_sei` e os `oferecidos` da última resposta (3 consultas, versão anterior) | **PASS** (recusa correta) |
| Termo alargado dentro do órgão (consulta 2) achando a unidade | **NOT_RUN**: nenhuma das duas precisou dele |
| Quatro consultas da ordem do Todos ao vivo | **NOT_RUN**: o caso de 4 consultas é o de uma inexistente, já visto com 3 |
| Corte em 50 com "Todos" | **NOT_RUN**: nenhuma resposta com 50 itens |
| `Location` `http://` do 302 de escrita | **NOT_RUN**: exige `--confirmar` |

Nenhum 403 nem `bloqueio_de_seguranca_da_instalacao`: 14 pedidos em três
comandos, de 8 a 14 s cada.

## Leitura

Com o órgão do prefixo, a sigla inteira acha a unidade no GO na primeira
consulta (`CBM`, `DETRAN`): o filtro funciona, e a quebra na `/` medida
pelo Todos é a da busca em "Todos". `ABC/GECONT-18141` não foi achado
porque não existe (Todos); a primeira versão deste relatório o marcou
como FAIL, corrigido aqui.
