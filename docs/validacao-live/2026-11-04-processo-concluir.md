---
type: SEIObservation
title: Ensaio live de processo concluir e plano da escrita
description: Prévia somente leitura de ies processo concluir na UFRJ 4.1.5 (PASS) e plano da escrita live para autorização do proprietário (NOT_RUN), 04/11/2026
---

# Ensaio live de `processo concluir` — 04/11/2026

Contrato em [processo concluir](../../contracts/cli/processo-concluir.md).
Binário em release do ramo `slice-concluir` (commit `5e9dc8d`), sessão já
aberta da UFRJ 4.1.5 de treinamento (conta pública `servidor1`, unidade
PR-P), índice da sessão. **Nenhuma escrita**: o proprietário ainda não
autorizou concluir processo na conta de treinamento compartilhada.

## Resultado

| Item | Instalação | Veredito |
| --- | --- | --- |
| Prévia (forma `tela`, uma unidade) | UFRJ 4.1.5 | **PASS** |
| Prévia (forma `tela`, oito unidades) | UFRJ 4.1.5 | **PASS** |
| Prévia, saída humana | UFRJ 4.1.5 | **PASS** |
| `--confirmar` (escrita e verificação) | todas | **NOT_RUN** (sem autorização) |
| Forma `funcao` ao vivo | UnB 4.0.9 | **NOT_RUN** |

Execuções (horários UTC; nenhuma URL, cookie ou `infra_hash` registrado):

1. 18:04:59 `ies processo concluir 99079.900901/2026-02 --json` → saída 0,
   `forma: "tela"`, `unidade: "PR-P"`, `aberto_em: ["PR-P"]`,
   `depois_esperado: []`, `confirmado: false`, `alterado: false`,
   `requisicoes: 2` (link de trabalho guardado no índice: tela do processo
   e árvore). A barra servida publicou o link `procedimento_concluir`
   assinado e a árvore, a situação na forma lida por `ies processo NUP`.
2. 18:05:13 `ies processo concluir 99079.900023/2025-36 --json` → saída 0,
   `forma: "tela"`, `aberto_em` com oito unidades (`ADCENTRAL`,
   `ADCENTRAL/GR`, `ADCENTRAL/GR/ARQUIVO`, `ADCENTRAL/SUPERTIC`,
   `DECANIA/CCS`, `DECANIA/CFCH`, `DECANIA/CT`, `PR-P`) e
   `depois_esperado` sem `PR-P`; `requisicoes: 2`. Confirma, ao vivo, a
   forma plural com siglas com `/` e a sigla da sessão igual à de
   `lnkInfraUnidade`.

3. 18:06:11 `ies processo concluir 99079.900901/2026-02` (saída humana) →
   saída 0: "Processo 99079.900901/2026-02 na unidade PR-P", "Aberto em:
   PR-P", "Ação na barra: tela", "Depois de concluir: (nenhuma unidade)",
   "Para concluir, repita com --confirmar", "Requisições: 2".

**Desvio do ritmo:** entre as execuções 1 e 2 passaram 11 s, abaixo dos
22 s combinados (erro de operação do agente; nenhuma recusa, bloqueio ou
efeito observado). As seguintes respeitaram o intervalo.

Efeito das prévias: só GETs da abertura (`procedimento_trabalhar` e
`procedimento_visualizar`), os mesmos de `ies processo NUP`, em processos
já visualizados na unidade. Nenhuma tela de conclusão foi pedida.

## Plano da escrita live (para autorização)

Objetivo: PASS de `--confirmar` nas duas formas, com a verificação por
`aberto_em` (critério de aceite da issue: UnB e PEN1 em processo criado
para o ensaio).

**Processos.** Um processo **criado para o ensaio** em cada instalação
(pelo navegador ou por quem o proprietário indicar; o IES não cria
processo), aberto só na unidade da sessão, sem documentos assinados de
terceiros. Não usar processos do acervo compartilhado.

| Ordem | Instalação | Forma esperada | Por quê |
| --- | --- | --- | --- |
| 1 | PEN1 5.0.4 (ou UFRJ 4.1.5) | `tela` (POST) | a forma de cinco instalações; a da PEN1 5.0.4 ainda não foi observada |
| 2 | UnB 4.0.9 | `funcao` (o GET é a escrita) | única instalação 4.0 hoje |

**Orçamento por instalação** (≥ 22 s entre comandos, nenhum retry, no
máximo 5 comandos):

1. `ies processo concluir NUP --json` (prévia; 2–3 GET). Seguir só se
   `forma` for a esperada, `aberto_em == [unidade]` e saída 0.
2. `ies processo concluir NUP --confirmar --json` (a escrita): `tela` =
   abertura + 1 GET + 1 POST + 1 GET; `funcao` = abertura + 3 GET.
3. `ies processo NUP --sem-andamentos --sem-propriedades --sem-marcadores --json`
   (leitura independente; 1–2 GET).
4. `ies processo concluir NUP --confirmar --json` de novo: deve sair 0 com
   `alterado: false` e `forma: null`, **sem escrita** (só a abertura).

**PASS** se: (2) sai 0 com `confirmado: true`, `alterado: true`,
`reconciliado_por_leitura: false`, `depois == depois_esperado == []` e
`requisicoes` igual ao orçamento; (3) mostra `aberto_em: []`
("Processo não possui andamentos abertos.") e a barra sem a ação de
concluir; (4) sai 0 sem escrita. **FAIL** com o subcódigo: saída 1
(`conclusao_nao_aplicada` ou recusa da tela/função antes da escrita).
**Saída 3**: parar; não repetir; ler com `ies processo NUP`, registrar
`falha_do_envio` e `verificacao` e reportar.

**Reabrir depois.** Observado nas sete instalações (11 gravações): o
processo concluído oferece `reabrirProcesso()` na barra, com a URL
`procedimento_reabrir` no corpo da função na tela do nó; o GET reabre na
unidade da sessão e a situação seguinte mostra a unidade **atribuída** ao
usuário. O IES não implementa reabrir (é outro comando da CLI pública; a
peça de leitura da função já existe e serviria). Para o ensaio, o
proprietário reabre pelo navegador ("Reabrir Processo" na barra) ou
autoriza uma fatia `ies processo reabrir NUP [--confirmar]` com o mesmo
padrão. Um processo criado para o ensaio pode simplesmente ficar
concluído.
