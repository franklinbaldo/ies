---
type: SEIObservation
title: Replay de conformidade da pesquisa avançada e rápida
description: Leitura offline das respostas gravadas de pesquisa de protocolo nas sete instalações públicas
---

# Replay da pesquisa no acervo público

A leitura usa os ZIPs do [catálogo de cassettes](../cassettes.md) e as
mesmas funções do IES que interpretam as telas e os destinos de trabalho.
O replay confere os hashes do ZIP e do membro, o esquema e o host.
Nenhuma requisição ao SEI faz parte da execução.

A pesquisa avançada reaproveita a sequência Controle, GET da tela, POST
e interpretação da resposta. A barra de resultados registra o total
declarado, a faixa observada e as linhas ausentes. Na pesquisa rápida,
o 302 gravado é interpretado usando a URL gravada do POST para conferir
o sistema e a unidade. O replay nunca abre o destino.

```sh
IES_ACERVO_CACHE=/caminho/dos/zips IES_ACERVO_SAIDA=saida.md \
  cargo test --locked --bin ies pesquisa_web::acervo::replay_do_acervo_inteiro -- --ignored
```

O corpo do POST original não consta das gravações. O replay demonstra
a leitura da resposta e o contexto gravado, com o alcance especificado
no [replay da pesquisa avançada](2026-11-04-processo-buscar-acervo.md).

## Resultado

**PASS.** Replay offline em 109 cassettes verificadas, com 688 leituras.
O teste terminou em 55,72 s com Rust 1.99.1 e perfil de teste. Os 14
testes sintéticos do módulo também passaram. RED não observado.

| Leitor | Conferências | Lido | Recusado | Não aplicável |
|---|---:|---:|---:|---:|
| Controle, unidade e link | 52 | 52 | 0 | 0 |
| GET da pesquisa, POST em processos derivado | 128 | 128 | 0 | 0 |
| Resposta do POST, pesquisa em processos | 66 | 39 | 1 | 26 |
| Plano da página 2 em processos | 14 | 14 | 0 | 0 |
| POST da página seguinte em processos | 14 | 0 | 0 | 14 |
| GET da pesquisa, POST em documentos derivado | 128 | 128 | 0 | 0 |
| Resposta do POST, pesquisa em documentos | 66 | 37 | 1 | 28 |
| Plano da página 2 em documentos | 26 | 25 | 1 | 0 |
| Página seguinte em documentos | 14 | 14 | 0 | 0 |
| 302 da pesquisa rápida | 180 | 180 | 0 | 0 |
| **Total** | **688** | **617** | **3** | **68** |

Os 180 redirecionamentos aceitos da pesquisa rápida se distribuem
entre UnB (86), UFRJ (58), GO (10), PEN1 (6), PEN2 (6), PEN3 (8) e
PEN4 (6). O catálogo contém também 21 respostas 200 da pesquisa
rápida, fora do recorte do leitor de 302.

Na pesquisa avançada, as 39 respostas lidas pelo leitor de processos
incluem 22 redirecionamentos, 14 listas e 3 avisos de “sem resultado”
em pesquisas gravadas com a opção documentos. As cassettes não trazem
o aviso equivalente com a opção processos. Em Goiás, o passo 4 de
`processo-busca-go--a7182b2ab8d7` declarou 966 resultados e faixa 1–10,
com 9 linhas presentes e uma ausente.

## Recusas e limites

- Duas leituras da mesma resposta na UnB, `documento-numero-e-busca-unb--f1b386d8e7bc#11`, retornaram
  `resposta_da_pesquisa_nao_reconhecida`. O POST devolveu o formulário sem lista nem aviso.
- Uma leitura do plano da página 2 em documentos na UnB,
  `documento-busca-autocompletar-unb--3dfc2c884999#9`, retornou
  `assinante_oculto_preenchido_na_pesquisa_em_documentos`.
- Os 68 casos não aplicáveis incluem 26 listas gravadas em documentos
  para o leitor de processos, 14 POSTs de página seguinte sem corpo
  gravado e 28 respostas de processos avaliadas pelo leitor de
  documentos.
- Os leitores de pesquisa avançada mantiveram as recusas dos relatórios
  anteriores. A pesquisa rápida não acrescentou recusas. Sem ensaio
  live nesta rodada.
