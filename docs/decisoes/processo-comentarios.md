---
type: ImplementationDecision
title: Parser tipado e conferência estrita dos comentários do processo
operation: processo.comentarios
implementation: partial
status: adopted-for-scope
scope: leitor offline 4.x/5.0, CLI GET e confirmação única sintética em treinamento; SQLite v33 e anti-IES
selected_strategy: web-formulario-servido-tabela-versao-conferencia-estrita
alternatives: web-formulario-servido-tabela-versao-conferencia-estrita,web-html-sem-identidade,rest-wssei
evidence_level: todos-323eb01-cassettes-estrutura-unb-go-testes-sinteticos-cli-https-sqlite
live_validation: not-run
---

# Comentários do processo — leitura CLI e anti-IES (#257)

## Contexto e observação

O Todos `comentarios.py` descreve dois formatos observados:
SEI 4.x com comentário em uma linha de dados mais `trComentario`;
SEI 5.0 GO com coluna Texto na própria linha.
As cassettes públicas de UnB, UFRJ, GO e PEN1–PEN4 documentam
`comentario_listar` e `comentario_cadastrar`, mas o card sozinho
não substitui o replay integral do HTML.

## Alternativas consideradas

1. **Leitor estrutural tipado + escrita conferida** (escolhida):
   `tl` para aninhamento e `html_atributos` para atributos,
   rejeitando ambiguidade; id, legenda e texto exato após a escrita.
2. **Aceitar HTML sem identidade ou total:** descartada, pois ausência de
   tabela não pode significar ausência de comentários e páginas truncadas
   não permitem provar a operação.
3. **REST/WSSEI:** descartada, não é suporte comum às instalações.

## Comparação

| Critério | Leitor estrutural tipado | HTML sem identidade | REST/WSSEI |
| --- | --- | --- | --- |
| Adere à interface Web observada | sim | parcial | não comprovado |
| Recusa lista truncada ou duplicada | sim | não | não demonstrado |
| Requer backend alternativo | não | não | sim |
| Escrita atestada por leitura | implementada no ensaio sintético | não | não |

## Decisão

Preservar o leitor que funciona nas formas 4.x/5.0 e ligá-lo ao GET
`comentario_listar` **oferecido na barra conferida da árvore**, por
`processo comentarios`. A CLI não monta URLs; POST só é enviado em `processo comentar --confirmar` sob guarda de treinamento. Não aceita
fotografia incompleta ou ausente como lista completa. A projeção do
anti-IES usa SQLite v33, distinguindo explicitamente lista desconhecida,
lista conhecida e total de observações maior que a página. Os testes
reais da CLI passam por HTTPS local, simulador e SQLite.

O verificador pós-escrita recebe exclusivamente uma lista relida completa
e exige id aceito e texto unívocos. A CLI só envia POST após conferir
o formulário publicado, recebe o 302 sem segui-lo e relê a lista.
Desvio após a tentativa produz saída 1, sem repetir a escrita. A capacidade
continua restrita a origens de treinamento; produção é recusada
antes da primeira requisição. A confirmação ainda não passou pelo replay
semântico bilateral das cassettes nem por ensaio live.
Não restaurar a visão P/T após uso: a preferência de sessão permanece
como o SEI a deixou, conforme a política do IES, evitando requisições
extras e alteração da sessão compartilhada.

## Evidências verificáveis

- [Contrato](../../contracts/cli/processo-comentarios.md)
- [Card da UnB](../../cassettes/unb/comentarios-ciclo-unb--c7d0c8a53b5e.md)
- [Testes do leitor](../../src/processo_comentarios/tests.rs)
- [Testes de conferência](../../src/processo_comentar/tests.rs)
- [Matriz de evidências](../../contracts/evidencias/processo-comentarios.md)
- [Teste da CLI (entrada inválida sem sessão)](../../tests/cli_processo_comentarios.rs)
- [Ciclo completo HTTPS ↔ anti-IES ↔ SQLite](../../tests/aies_comentarios_cli.rs)
- [Inspeção estrutural de 76 interações UnB/GO](https://github.com/franklinbaldo/ies/actions/runs/42786966461)
- [Live](../validacao-live/2026-11-05-processo-comentarios.md)

## Limitações e revisão

Revisar após replay dos HTMLs 4.x/5.0 e primeira leitura real da UnB:
identidade do título, transição 302 de lista vazia, paginação e
orçamento de requisições. Não considerar a confirmação sintética evidência de escrita em SEI externo.
