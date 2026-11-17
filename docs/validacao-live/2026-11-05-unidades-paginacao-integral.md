---
type: SEIObservation
title: Paginacao integral da lupa e ensaios somente leitura UnB e GO
description: Issue #244, PR #301; 274 paginas sinteticas testadas no CI e falhas tipadas observadas ao vivo sem repetir pedidos
---

# #244 — paginacao integral da lupa (retomada de 05/11/2026)

## Observacao anterior aos testes

A cassette GO `cassettes/go/unidade-listar-destinos-go--965da599b3e4.md`
documenta o primeiro GET da lupa com **13.671 unidades em 274 paginas**,
mas so seis respostas ao todo (3 GET, 3 POST) foram gravadas. O
[replay GO obrigatorio](../../.github/workflows/ci.yml) nao cobre as
274 paginas. A referencia de comportamento e o Todos,
`tests/test_lupa_unidades_010.py` @ `323eb01`.

## Testes da nova fatia, so com HTML sintetico

Em `src/unidade_lupa/fluxo_testes.rs`, `RoteiroPaginacaoIntegral`
simula **4 GET de navegacao + 274 POST de leitura = 278 pedidos**,
com indices 0 a 273, 13.671 IDs unicos e ultima pagina curta.
O teste confere total, primeira e ultima identidade, indices
solicitados e ausencia de POST extra. Outro teste adultera o
filtro de sigla **somente na pagina 274** e exige falha fechada
`FormularioInesperado`, sem devolver lista parcial nem retry.

Os testes caracterizam a implementacao preexistente: **RED novo
nao observado**; nao inventar contagem. Evidencias RED historicas
da issue #244: [3 falhas no CI 42063259438](https://github.com/franklinbaldo/ies/actions/runs/42063259438)
e [1 falha no CI 42134942869](https://github.com/franklinbaldo/ies/actions/runs/42134942869).
A primeira revisao desta fatia falhou em Rustfmt e Clippy
([CI 42142445906](https://github.com/franklinbaldo/ies/actions/runs/42142445906));
as correcoes preservaram os testes.
[CI 42142988367](https://github.com/franklinbaldo/ies/actions/runs/42142988367)
aprovou 3/3 jobs (prek, binario Release, replay GO), com
os dois testes sinteticos na suite Rust. Build local/RED local: **NOT_RUN**
(sem Rust e sem DNS externo no sandbox).

## Validacao autenticada em treinamento, somente leitura

Foram usados os arquivos privados 0600, contas publicadas pelas
instalacoes e executores Actions temporarios, **sem expor valores de
cookie, senha ou URL assinada**. Os workflows temporarios foram
retirados do branch apos os ensaios. Nenhum pedido foi repetido
apos falha.

| Ambiente / etapa | Resultado | Pedidos observaveis |
| --- | --- | --- |
| UnB, `ies login --json` | **PASS**, sessao de treinamento | **5** |
| UnB, `unidade listar --atualizar --json` | **FAIL**, `link_da_lupa_ambiguo` | **contagem nao publicada pela CLI na falha** |
| UnB, `unidade pesquisar a --json` | **NOT_RUN**, coleta anterior falhou | **0** |
| GO, `ies login --json` | **PASS**, sessao de treinamento | **6** |
| GO, `unidade listar --atualizar --json` | **FAIL**, `formulario_da_lupa_inesperado` | **contagem nao publicada pela CLI na falha** |
| GO, `unidade pesquisar a --json` | **NOT_RUN**, coleta anterior falhou | **0** |

Registros dos ensaios:
[UnB, run 42143302555](https://github.com/franklinbaldo/ies/actions/runs/42143302555)
e [GO, run 42143615203](https://github.com/franklinbaldo/ies/actions/runs/42143615203).
Os jobs Actions terminaram com sucesso **operacional**, pois o
roteiro registrou FAIL e nao encerrou o job com falha; isso **nao**
significa PASS da listagem. Nao inferir uma contagem HTTP exata
para a operacao recusada. A gravacao do snapshot nao foi atestada:
a leitura integral recusou a tela. **Nenhuma escrita de dominio
no SEI; sessao de login de treinamento criada nos dois ambientes.**
Nenhum resíduo de documento/processo/unidade foi criado ou alterado
pelo IES nesta execucao.

## Pendencias para fechar a issue

- Investigar **por observacoes ja disponiveis** (cassettes/capturas
  preservadas) por que a UnB apresentou multiplos links elegiveis e
  GO apresentou formulario inesperado. Nao repetir as requisicoes
  que falharam, nem relaxar validacao de URLs ou formularios.
- Corrigir a divergencia somente apos evidencia estrutural e
  RED, garantindo a mesma protecao de origem, assinatura, filtros
  e integralidade.
- Registrar orcamento inclusive nas **falhas** da lupa,
  sem vazar dados privados, para medir a coleta autenticada.
- Concluir coleta integral de UnB e GO, pesquisa offline 0 HTTP
  e teste que captura stdout/stderr reais da CLI de sugestoes.
  Nenhuma dessas condicoes foi comprovada nesta execucao.

Issue continua aberta, reserva `80 🦊`; PR #301 fica Draft.
