---
type: Contract
title: Catálogo de marcadores da unidade
description: Leitura da página publicada de marcadores da unidade por GET, distinta da atribuição ao processo
---

# `ies marcador listar [--inativos]` (#880)

Lê a página de catálogo misto publicada pelo menu da unidade ativa.
Ativos por padrão; `--inativos` seleciona inativos na mesma página.
Não percorre outras páginas, não alterna/restaura filtro persistente e não cria,
altera, desativa, reativa, exclui ou aplica marcadores.

- JSON: unidade, origem, requisições, total_declarado, total_observado,
  estado solicitado, faixa_observada (omitida quando vazio comprovado) e itens.
  Cada item conserva ID decimal como string, nome, cor e estado observado.
  Texto humano distingue a faixa, o estado e o rótulo ID#NOME.
- A seleção vazia em página parcial não prova ausência no catálogo inteiro.
  Tabela ausente, estado contraditório e legenda incoerente são erros.
- A unidade é a reconhecida no Controle; se a página publicar outra unidade,
  a leitura falha. O link deve ser publicado e inequívoco, na mesma instalação.
  Uma janela sem barra própria é vinculada ao link do Controle que a abriu.
- A família com filtro persistente S/N não é suportada nesta fatia.
  Nenhum POST de catálogo é realizado.

## Evidência e limites

Oráculo [Todos](https://github.com/franklinbaldo/todos/blob/caabe699bcf13b61b12dc5863638c8d90c0155c1/src/todos/marcadores.py);
[decisão R251](https://github.com/franklinbaldo/ies/issues/880#issuecomment-7179497205).
Os cards [UnB](../../cassettes/unb/marcador-listar-estados-unb--c47ad7e28f89.md)
e [PEN1](../../cassettes/pen1/marcador-listar-estados-pen1--8358611b6263.md)
fundamentam os replays de leitor e navegação; o E2E loopback do binário usa
páginas sintéticas. Não equivale a prova de compatibilidade live ou a
reconstrução de unidade para cada captura histórica.

Persistência de catálogo de gestão e rota anti-IES baseada em testemunhas
SQLite ficam fora desta entrega de leitura (R296). A atribuição existente
não é fonte de estado ativo/inativo. Sem migration nova.
