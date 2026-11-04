---
type: SEIObservation
title: Replay de conformidade do acervo público pelos leitores de processo concluir
description: Ação da barra, função concluirProcesso, tela Conclusão de Processo, resposta da escrita e regra de verificação por aberto_em nas 190 cassettes com procedimento_concluir, offline, 04/11/2026
---

# Replay do acervo — processo concluir — 04/11/2026

As respostas gravadas das 190 cassettes públicas que registram
`procedimento_concluir` passaram pelos **mesmos leitores** de
`ies processo concluir`, offline
(`cargo test --bin ies replay_de_processo_concluir -- --ignored`, ZIPs em
`IES_ACERVO_CACHE`, cada um conferido pelo card). Nenhuma requisição ao
SEI. Contrato em [processo concluir](../../contracts/cli/processo-concluir.md).

Leitores: ação de concluir na barra de cada árvore
(`procedimento_visualizar`); `concluirProcesso()` em cada tela do nó
(`arvore_visualizar`); a tela "Conclusão de Processo" conferida e o POST
derivado pelo primitivo; o 302 do POST; a resposta do GET da forma
`funcao`; e, por escrita, a regra de verificação: a unidade de
`lnkInfraUnidade` da última tela do processo está no `aberto_em` lido
antes e não está no da primeira árvore do mesmo processo depois.

| Leitor | Classe | unb | ufrj | go | pen1 | pen2 | pen3 | pen4 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| ação na barra | `funcao` | 290 | 0 | 0 | 177 | 0 | 0 | 0 |
| ação na barra | `tela` | 0 | 304 | 202 | 0 | 185 | 176 | 184 |
| ação na barra | não oferece | 40 | 46 | 40 | 38 | 34 | 36 | 37 |
| função na tela do nó | URL lida e conferida | 104 | 0 | 0 | 56 | 0 | 0 | 0 |
| função na tela do nó | `funcao_de_concluir_ausente_na_tela_do_no` | 0 | 88 | 39 | 0 | 31 | 27 | 34 |
| resposta da escrita | 302 esperado | 0 | 36 | 32 | 0 | 28 | 28 | 29 |
| resposta da escrita | HTML 200 (escrita pela função) | 34 | 0 | 0 | 32 | 0 | 0 | 0 |
| tela de conclusão | conferida e derivada (7 campos) | 0 | 36 | 32 | 0 | 28 | 28 | 29 |
| verificação | verificada (nenhuma unidade aberta) | 30 | 33 | 30 | 29 | 26 | 25 | 27 |
| verificação | verificada (outras unidades seguem abertas) | 4 | 3 | 2 | 3 | 2 | 3 | 2 |

Leitura: nenhuma recusa. As 219 escritas gravadas (153 POST, 66 GET da
função) teriam sido aceitas como resposta esperada e verificadas pela
situação. "Não oferece" são as árvores do processo já concluído (ou de
processo noutra unidade). Nas instalações da forma `tela`, a tela do nó
não define `concluirProcesso()`: a ausência confirma que lá a função não é
o caminho. O replay não prova o corpo do POST (não gravado), só que o
derivado tem os 7 campos da forma observada.
