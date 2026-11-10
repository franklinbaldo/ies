---
type: SEIObservation
title: Ensaio live de processo enviar (prévia) e plano da escrita
description: Prévia somente leitura de ies processo enviar na UFRJ 4.1.5 (PASS, quatro comandos, quatro requisições cada, contadas no relay CONNECT local) e o plano da escrita com ida e volta entre duas unidades da mesma conta, que aguarda autorização (NOT_RUN), 04/11/2026
---

# Ensaio live de `processo enviar` — 04/11/2026

Contrato em [processo enviar](../../contracts/cli/processo-enviar.md).
Binário de depuração do ramo `slice-enviar`, sessão já aberta da UFRJ 4.1.5
de treinamento (conta pública `servidor1`, unidade PR-P), índice da
sessão. Requisições contadas por um relay CONNECT local (um túnel TLS por
requisição); 23 s ou mais entre comandos; nenhum retry. **Nenhum POST de
escrita.**

| Item | Instalação | Veredito |
| --- | --- | --- |
| Prévia, destino com um item exato entre vários da busca | UFRJ 4.1.5 | **PASS** |
| Recusa: sigla sem item exato (`oferecidos`) | UFRJ 4.1.5 | **PASS** |
| Recusa: processo já aberto no destino | UFRJ 4.1.5 | **PASS** |
| Prévia com `--manter-aberto`, saída humana, sigla em minúsculas | UFRJ 4.1.5 | **PASS** |
| `--confirmar` (POST, 302 e verificação) | — | **NOT_RUN** (aguarda autorização; plano abaixo) |
| Envio sem manter aberto (origem sai) | — | **NOT_RUN** (não gravado no acervo) |
| UnB 4.0.9 e PEN1 (pedidos na issue) | — | **NOT_RUN** |

Execuções (horários UTC; nenhuma URL, cookie ou `infra_hash` registrado):

| Hora | Comando | Saída | Túneis | Resultado |
| --- | --- | --- | --- | --- |
| 21:43:46 | `processo enviar 99079.900901/2026-02 --para PR-A --json` | 0 | 4 | `requisicoes: 4`; `unidade: PR-P`, `aberto_em: [PR-P]`, `destino: PR-A - Pró-Reitoria de Acesso` (id `110000938`), `depois_esperado: [PR-A]` |
| 21:44:13 | `… --para PR --json` | 1 | 4 | `destino_nao_oferecido_pelo_sei`; `oferecidos`: 10 itens (`PR-A`, `PR-A/SUPA`, `PR-OGT`, `PR-P/COPESS`, … e `Tramita.Gov.BR - Processo recebido via Tramita.GOV.BR`), sem a própria PR-P |
| 21:44:39 | `processo enviar 99079.900023/2025-36 --para ADCENTRAL --json` | 1 | 4 | `processo_ja_aberto_no_destino`, `aberto_em` com as oito unidades |
| 21:45:05 | `processo enviar 99079.900901/2026-02 --para pr-a --manter-aberto` | 0 | 4 | "Destino: PR-A - Pró-Reitoria de Acesso", "Depois de enviar: PR-P, PR-A", "Para enviar, repita com --confirmar", "Requisições: 4" |

Cada execução: tela do processo pelo link de trabalho guardado (~42 KB),
árvore (~29–37 KB), tela "Enviar Processo" (~30 KB) e a consulta do
autocomplete (~6 KB, XML). É o orçamento do contrato com a abertura
quente: 3 GET + 1 POST (consulta). Fatos novos: a busca da UFRJ por `PR`
casa siglas por prefixo e exclui a unidade da sessão (`unidade_atual=0`);
a busca exata por `PR-A` devolve também `PR-A/SUPA` e `PR-A/SUPA/SEC`, e a
escolha pela sigla exata isolou `PR-A`.

## Plano da escrita (para autorização)

**Por que é delicado.** O envio não tem desfazer na barra. Sem manter
aberto, o processo sai da unidade da sessão; voltá-lo exige uma sessão na
unidade de destino. O IES ainda não troca de unidade.

**O que torna a volta possível na UFRJ.** A conta `servidor1` tem
permissão em oito unidades (tela `infra_trocar_unidade` gravada no acervo,
`processo-pasta-mista-ufrj`: "Lista de Unidades com Permissão (8
registros)": ADCENTRAL, CAXIAS, DECANIA, FCC, MACAE, PR-A, PR-OGT, PR-P).
A mesma pessoa pode receber em PR-A e devolver a PR-P, trocando a unidade
no navegador (o seletor de unidade do topo).

**Processo.** `99079.900901/2026-02` (já usado no ensaio de concluir,
aberto só em PR-P) ou um processo novo criado no navegador para o ensaio.

**Passos** (UFRJ, ≥ 22 s entre comandos, nenhum retry):

1. `ies processo enviar NUP --para PR-A --manter-aberto --confirmar --json`:
   esperado `alterado: true`, `depois: [PR-A, PR-P]`, 6 requisições
   (2 GET abertura, 1 GET tela, 1 POST consulta, 1 POST envio, 1 GET
   verificação — 4 GET + 2 POST com a abertura quente). Mede o 302 e a
   verificação na forma gravada no acervo.
2. `ies processo NUP --json`: confirma `aberto_em: [PR-A, PR-P]` (1–2 req.).
3. `ies processo enviar OUTRO_NUP --para PR-A --confirmar --json` (sem
   manter aberto, processo criado para isso): esperado `depois: [PR-A]`;
   mede o efeito que o acervo não tem (a origem sai).
4. **Volta, pelo proprietário** no navegador: trocar para PR-A, abrir os
   dois processos; no de 3, "Enviar Processo" para PR-P; no de 1,
   "Concluir" em PR-A (ou devolver a PR-P também). Alternativa sem
   navegador: autorizar uma issue `ies unidade trocar SIGLA` antes do
   passo 3.
5. `ies processo NUP --json` nos dois: confirma o estado final (só PR-P).

Se o passo 1 sair com 3 (efeito incerto), parar: conferir com
`ies processo NUP` antes de qualquer outro comando. A UnB 4.0.9 (Unidade1 →
Unidade2, gravado pelo Todos) pode repetir os passos 1–2 se a conta
`aluno1` tiver as duas unidades; não verificado neste ensaio.
