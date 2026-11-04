---
type: SEIObservation
title: Ensaio live de processo concluir e processo reabrir
description: Prévia de ies processo concluir na UFRJ 4.1.5 (PASS) e, autorizada pelo proprietário, a escrita de concluir e de reabrir na UFRJ 4.1.5 (forma tela, URL por variável) e na UnB 4.0.9 (forma função), com a contagem de requisições e a verificação por leitura (PASS), 04/11/2026
---

# Ensaio live de `processo concluir` e `processo reabrir` — 04/11/2026

Contratos em [processo concluir](../../contracts/cli/processo-concluir.md)
e [processo reabrir](../../contracts/cli/processo-reabrir.md). Duas
partes: a prévia somente leitura (18:04–18:06, binário `5e9dc8d`) e, depois
da autorização do proprietário ("Autorizo concluir, escolher vc"), a
escrita (18:34–18:41, seção "Escrita autorizada").

## Prévia (antes da autorização)

Binário em release do ramo `slice-concluir` (commit `5e9dc8d`), sessão já
aberta da UFRJ 4.1.5 de treinamento (conta pública `servidor1`, unidade
PR-P), índice da sessão. Nenhuma escrita.

| Item | Instalação | Veredito |
| --- | --- | --- |
| Prévia (forma `tela`, uma unidade) | UFRJ 4.1.5 | **PASS** |
| Prévia (forma `tela`, oito unidades) | UFRJ 4.1.5 | **PASS** |
| Prévia, saída humana | UFRJ 4.1.5 | **PASS** |
| `--confirmar` (escrita e verificação) | UFRJ 4.1.5, UnB 4.0.9 | **PASS** (abaixo) |
| Forma `funcao` ao vivo | UnB 4.0.9 | **PASS** (abaixo) |
| Forma `tela` na PEN1 5.0.4 | PEN1 | **NOT_RUN** (fora deste ensaio) |

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

## Escrita autorizada: concluir e reabrir

Binário em release do ramo `slice-reabrir` (commit `c8f82a8`, `processo
concluir` mais `processo reabrir`), copiado para fora do repositório.
Sessões já abertas (nenhum `ies login`), índices das sessões. Cada
comando passou por um relay `CONNECT` local que registrou um túnel por
requisição; os túneis contados conferem com `requisicoes` em todos os
comandos. Intervalo de pelo menos 22 s entre comandos; nenhum retry do
IES. JSON de cada comando guardado em arquivo privado (0600), fora do
repositório; nenhuma URL, cookie ou `infra_hash` nas saídas (conferido).

Processos escolhidos pelo agente, de testes automatizados, abertos só na
unidade da sessão (ou em nenhuma) antes do ensaio: UFRJ
`99079.900901/2026-02` (a prévia anterior já o mostrava aberto só em
PR-P); UnB `23106.001574/2026-60` (processo gerado num ensaio anterior,
encontrado já concluído em todas as unidades).

### UFRJ 4.1.5 (conta `servidor1`, unidade PR-P)

| Hora | Comando | Túneis = `requisicoes` | Campos | Veredito |
| --- | --- | --- | --- | --- |
| 18:34:41 | `concluir` (prévia) | 2 | `forma: tela`, `aberto_em: [PR-P]` | PASS |
| 18:35:06 | `concluir --confirmar` | 5 (2 abertura, GET da tela, POST, 1 verificação) | `alterado: true`, `depois: []`, `reconciliado_por_leitura: false` | **PASS** |
| 18:35:34 | `processo NUP` | 1 | `aberto_em: []` | PASS |
| 18:35:58 | `concluir --confirmar` de novo | 2 | `alterado: false`, `forma: null`, sem escrita | PASS |
| 18:36:23 | `reabrir` (prévia) | 2 | `reabertura_oferecida: true`, `aberto_em: []` | PASS |
| 18:36:48 | `reabrir --confirmar` | 5 (2 abertura, tela do nó, GET de reabrir, 1 verificação) | `forma: variavel`, `alterado: true`, `depois: [PR-P]` | **PASS** |
| 18:37:16 | `processo NUP` | 1 | `aberto_em: [PR-P]`, atribuído a `servidor1` | PASS |

Estado final: aberto só em PR-P, atribuído a `servidor1` (o estado de
antes, mais a atribuição).

### UnB 4.0.9 (conta `aluno1`, Unidade1)

| Hora | Comando | Túneis = `requisicoes` | Campos | Veredito |
| --- | --- | --- | --- | --- |
| 18:37:26 | `concluir` (prévia) | 3 | `forma: null`, `aberto_em: []` (já concluído; não serve para concluir primeiro) | — |
| 18:38:03 | `reabrir` (prévia) | 2 túneis | saída 1, `falha_de_rede` no 2.º GET (o túnel recebeu 39 bytes em 10 s); leitura, nada a reconciliar | — |
| 18:39:08 | `reabrir` (prévia, nova execução) | 2 | `reabertura_oferecida: true`, `aberto_em: []` | PASS |
| 18:39:34 | `reabrir --confirmar` | 5 (2 abertura, tela do nó, GET de reabrir, 1 verificação) | `forma: funcao`, `alterado: true`, `depois: [Unidade1]` | **PASS** |
| 18:40:02 | `processo NUP` | 1 | `aberto_em: [Unidade1]`, atribuído a `aluno1` | PASS |
| 18:40:26 | `concluir` (prévia) | 2 | `forma: funcao`, `aberto_em: [Unidade1]` | PASS |
| 18:40:51 | `concluir --confirmar` | 5 (2 abertura, tela do nó, GET da função, 1 verificação) | `alterado: true`, `depois: []`, `reconciliado_por_leitura: false` | **PASS** |
| 18:41:19 | `concluir --confirmar` de novo | 2 | `alterado: false`, `forma: null`, `aberto_em: []`, sem escrita | PASS |

Estado final: concluído em todas as unidades, como estava antes do
ensaio.

**Desvios do plano.** Na UnB a ordem foi invertida (reabrir, depois
concluir), porque o processo escolhido já estava concluído; assim o
estado final é o de antes. A prévia de reabrir das 18:38:03 falhou na
rede e foi executada de novo 65 s depois, à mão (é leitura: não há efeito
a reconciliar); por isso a UnB usou 8 comandos e a leitura independente
depois de concluir foi a abertura do `concluir --confirmar` repetido
(`aberto_em: []`), não um `processo NUP` à parte.

**Leitura.** As duas formas de concluir e as duas formas de a tela do nó
publicar reabrir funcionaram ao vivo como nas gravações: a resposta da
escrita foi a esperada nos quatro casos (302 do POST; HTML 200 da função
e dos dois GET de reabrir), sem reconciliação. A reabertura atribuiu o
processo ao usuário nas duas instalações, como nas 11 gravações.
