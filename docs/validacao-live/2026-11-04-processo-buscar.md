---
type: SEIObservation
title: Ensaio live de processo buscar na PEN3 5.0.4
description: PASS/FAIL/NOT_RUN de ies processo buscar na PEN3 (NUP na caixa, NUP fora da caixa, inexistente, parcial, --pagina 2, fora do intervalo, termo só de dígitos e zero à esquerda; termo só de dígitos contra o Cloudflare), 04/11/2026
---

# Ensaio live de `ies processo buscar` — PEN3 5.0.4 — 04/11/2026

Primeiro ensaio do comando na PEN3 (`sei.orgao3.tramita.processoeletronico.gov.br`,
SEI 5.0.4), com o binário release do ramo `slice-buscar` (commit
`e71b23e`, antes da correção do bloqueio da borda). Complementa o
[ensaio de 03/11](2026-11-03-processo-buscar.md) (UnB, GO, PEN1) e o
[replay do acervo](../validacao-replay/2026-11-04-processo-buscar-acervo.md).

## Condições

- **Sessão:** a de `ies login` com a conta pública de treinamento da PEN3
  ([credenciais](../interfaces/credenciais-de-treinamento.md)), unidade
  `TESTE`, arquivos `0600` numa pasta `0700` fora do repositório; não
  precisou relogar.
- **Efeito avaliado:** GET do Controle e da tela de Pesquisa são leitura;
  o POST da pesquisa lembra página e filtro na conta
  (`PaginaOuFiltroLembrado`); o 302 para `procedimento_trabalhar` **não**
  é seguido, então nenhum processo é aberto nem recebido.
- **Orçamento declarado:** até 20 comandos, ≥ 20 s entre eles, sem
  retry; 3 requisições na página 1 e 4 na página N. Usados: 8 comandos
  (16:48–16:52 UTC, intervalos de 28 a 44 s) e 25 requisições.
- **Termos.** Os NUPs são conteúdo público da instalação de treinamento:
  `98390.000038/2026-88` está no índice local (colhido do Controle);
  `98390.000039/2026-22` aparece na cassette pública
  `processo-busca-pen3` (unidade `TESTE`) e **não** está entre os 294
  NUPs que o índice colheu da caixa, nem entrou depois do GET do Controle
  deste ensaio. Nenhum id, cookie, URL assinada ou HTML foi versionado.

## Resultado

| # | Termo | Esperado | Saída do IES | Req. | Veredito |
| --- | --- | --- | --- | ---: | --- |
| 1 | NUP na caixa | `processo_unico` | `processo_unico`, id igual ao do índice local | 3 | **PASS** |
| 2 | NUP fora da caixa | 302 ou lista | `processo_unico` | 3 | **PASS** |
| 3 | `99999.999999/2026-00` | resposta observada | `sem_resultado`, saída 0 | 3 | **PASS** |
| 4 | `98390` (parcial) | `lista` | 10 linhas, total 81, faixa 1–10 | 3 | **PASS** |
| 5 | `98390 --pagina 2` | faixa 11–20 | 10 linhas, mesmo total 81, faixa 11–20 | 4 | **PASS** |
| 6 | `98390 --pagina 100000` | recusa antes do POST de página | `pagina_fora_do_intervalo`, saída 1 | 3 | **PASS** |
| 7 | `98390000039202622` (só dígitos) | resposta do SEI | 403 da borda; saiu `sessao_nao_autorizada` | 3 | **FAIL → corrigido offline** |
| 8 | `098390.000039/2026-22` (zero à esquerda) | termo opaco, sem normalizar | `sem_resultado`, saída 0 | 3 | **PASS** |

## Observações

- **NUP fora da caixa (#58).** O caso 2 é o que a issue pede: um NUP que
  o Controle da unidade não lista foi localizado pela Pesquisa (302 para
  o processo, interpretado sem seguir). O comando publica só o `id`; abrir
  o processo continua com `ies processo NUP`.
- **NUP opaco.** Com o zero à esquerda (caso 8), o SEI não achou nada, e o
  IES enviou o termo como digitado: não há normalização nem heurística de
  identidade. `sem_resultado` continua sendo resposta observada, não
  `processo_inexistente`.
- **Bloqueio da borda (caso 7).** O termo só de dígitos fez a borda da
  instalação (Cloudflare) responder 403 ao POST, com o título "Attention
  Required! | Cloudflare" e "Sorry, you have been blocked". O IES não
  repetiu nada, mas classificou como `sessao_nao_autorizada`, o que sugere
  relogin sem necessidade: o caso 8, 28 s depois, passou com a mesma
  sessão. Corrigido no commit seguinte (a página passa a ser
  `bloqueio_de_seguranca_da_instalacao`, teste sintético); a reprodução
  live foi feita depois, com autorização (seção seguinte): o termo não é
  a causa.
- **Lista e paginação.** A barra veio "Exibindo 1 - 10 de 81" e a página 2,
  pelo POST de `navegar()`, manteve o total; nenhuma linha de acerto em
  documento apareceu nessas duas páginas.

## Termos só de dígitos (Cloudflare)

Rodada autorizada pelo proprietário (18:24–18:41 UTC, commit `1cc78b2`,
≥ 60 s entre comandos, sem retry; tabela completa e contagem no
[relatório de documento buscar](2026-11-04-documento-buscar.md#termos-só-de-dígitos-cloudflare)).

| Início (UTC) | Termo | Túneis | Resposta |
| --- | --- | ---: | --- |
| 18:25:18 | `98390.000039/2026-22` (controle) | 3 | `processo_unico` |
| 18:26:37 | `98390000039202622` (o termo do caso 7) | 1 | 403 Cloudflare **no GET do Controle**: `bloqueio_de_seguranca_da_instalacao` |
| 18:37:30 | `98390.000038/2026-88` (controle, 10 min 53 s depois) | 3 | `processo_unico` |
| 18:38:46 | `98390000038202688` (só dígitos) | 3 | `processo_unico`, o mesmo id do controle |

O bloqueio voltou com o mesmo termo do caso 7, mas na requisição que
**não** leva o termo (o GET do Controle, primeira do comando); o POST não
saiu. Um NUP só de dígitos passou 76 s depois de um controle, e o SEI o
casou com o processo do NUP formatado. A hipótese de que o termo só de
dígitos dispara o Cloudflare **não se sustenta**: a borda bloqueia por
origem ou ritmo agregado, não pelo termo. O comando já publica o bloqueio
como `bloqueio_de_seguranca_da_instalacao`, sem retry.

## Não executado

- Sigiloso, restrito e processo de outra unidade sem acesso: a conta de
  treinamento não tem um NUP conhecido nessas condições (**NOT_RUN**).
- UFRJ, PEN2 e PEN4 continuam **NOT_RUN** (fora do escopo desta rodada).
