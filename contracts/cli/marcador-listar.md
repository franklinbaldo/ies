---
type: Contract
title: Catálogo de marcadores da unidade (proposta)
description: Primeira fatia de leitura do catálogo da unidade, distinta de marcadores aplicados ao processo
---

# `ies marcador listar [--inativos]` — proposta #880

**WIP.** Leitor experimental em `src/marcadores_unidade.rs`, ligado somente à suíte de testes. A CLI ainda não está disponível; não há aceite de compatibilidade.

- Abrir o link `marcador_listar` publicado pelo menu da unidade ativa, pelo fluxo de sessão e navegação existente.
- Listar ativos por padrão; `--inativos` seleciona inativos. JSON tipado e texto humano preservam ID decimal como string, nome, rótulo ID#NOME, cor e estado efetivamente observados. Não confundir ID de catálogo com identidade de uma atribuição.
- Tela mista: ler Nome, ID, Ícone e Ações pelo cabeçalho, não pela posição presumida. Ações de desativação/reativação distinguem estados no oráculo. Ausência de ambas não prova estado ativo; confirmar forma real antes de atribuir estado. Uma lista vazia reconhecida não equivale a uma tela não reconhecida ou incompleta.
- Tela com filtro persistente: o oráculo Todos usa `frmMarcadorLista`, `hdnMarcadoresFiltro`, S para ativos e N para inativos. A troca por POST pode alterar a visualização da sessão sem mutar o catálogo. **Não portar a restauração automática do Todos**: AGENTS.md determina não restaurar visão/página/colunas. [Decisão R251 do Elefante](https://github.com/franklinbaldo/ies/issues/880#issuecomment-7179497205): primeira fatia lê por GET o catálogo tal como publicado, não alterna/restaura filtro implicitamente. Se uma fatia posterior exigir troca persistente, declarar o efeito no contrato. Não prometer suporte à família filtrada até confronto dos originais.
- Nenhuma criação, alteração, desativação, reativação, exclusão ou aplicação de marcador nesta fatia.

## Evidência e testes pendentes

Oráculo comportamental Todos fixado em [caabe699](https://github.com/franklinbaldo/todos/blob/caabe699bcf13b61b12dc5863638c8d90c0155c1/src/todos/marcadores.py) e [leitor](https://github.com/franklinbaldo/todos/blob/caabe699bcf13b61b12dc5863638c8d90c0155c1/src/todos/sei/marcadores_lista.py). O leitor Python considera ativo o marcador sem ação Reativar, inclusive sem ações, e ignora linha sem ID/nome; não usar essa tolerância como prova de completude do SEI.

Cards [UnB](../../cassettes/unb/marcador-listar-estados-unb--c47ad7e28f89.md) (16 GET; 3 marcador_listar), [GO](../../cassettes/go/marcador-listar-estados-go--dccec894637f.md) (6 GET/2 POST; 6 marcador_listar). Metadados, não replay. Conferir os sete cards dedicados e corpos originais antes do aceite; comparar estado, ID, cor, vazio explícito e variantes de filtro. Reusar o catálogo SQLite existente quando a semântica coincidir; ele não armazena estado ativo/inativo atualmente. Não inferir inativos por ausência em catálogo de aplicação.

[Inspeção 45731754075](https://github.com/franklinbaldo/ies/actions/runs/45731754075): 7/7 cassettes e 61 interações verificadas por SHA-1/SHA-256; não é replay. Nas duas fixtures Todos, o ID textual coincide com o value do checkbox em 228/228 linhas; 114 IDs distintos por página, sem estado ambíguo. Homônimos são hipótese sintética. O leitor distingue ausência de tabela/filtro de vazio declarado e conserva total/faixa da página antes de qualquer filtro local. Testes sintéticos e `cassette_catalogo_misto_unb_pen1` escritos; o seletor `cassette_` da CI existente inclui este último. Rust, replay, CLI real, anti-IES/SQLite e live: NOT_RUN. Sem mudança de contrato executável até a fatia funcional e os gates SHA-exatos.
