---
type: Guide
title: Prova de scripts de navegação ativos para documentos internos
description: Decisão de segurança da issue 246 sobre escolha de tipo e filtro na interface Web
---

# Prova de navegação por script ativo

Complementa a [decisão documento criar](../decisoes/documento-criar.md).

## Navegação por funções JS ativas (06/11/2026, #246)

A escolha do tipo e o filtro `Exibir todos` têm um POST de navegação,
não de inclusão. A implementação anterior buscava a definição literal de
`escolher(id)` ou `exibirSeries(tipo)` no **HTML inteiro**. Uma cópia
da função em `template`, em `script type=application/json` ou em
comentário HTML bastava para autorizar o POST, embora o navegador não
executasse essa função. Isso é uma lacuna de prova de comportamento,
não evidência de exploração na instalação.

**Decisão:** continuar exigindo a sequência literal já observada no
Todos/UFRJ, mas contá-la apenas na fonte de `script` clássico inline
ativo, fora de `template`, sem `src`, conferido pelo DOM. Reusar o
extrator literal de scripts que já protege a criação, sem executar JS e
sem introduzir biblioteca/parser genérico. Mais de uma definição ativa
permanece recusa fechada. Casos positivos preservam a navegação legítima.

**Testes RED-first:** `tests_scripts_inertes.rs` cobre as três formas
inertes nos dois caminhos; controles positivos usam a mesma função ativa
da captura Todos. A correção altera somente a autorização do POST de
navegação, não o POST criador, o contrato de efeito ou a contagem da
confirmação. Nenhum ensaio vivo novo foi executado nesta fatia.

## Comentários e blocos JavaScript inertes (06/11/2026, #246)

A presença da sequência literal `function escolher(id)` ou
`function exibirSeries(tipo)` dentro de um `script` clássico não basta
quando ela aparece num **comentário JS**, numa **string/template literal**
ou num bloco subordinado (por exemplo `if (false) {...}`). A
implementação antiga compactava todos os caracteres do `script`,
inclusive os inertes, e podia autorizar um POST de navegação sem função
realmente publicada.

A guarda de navegação agora percorre lexicalmente a fonte servida e
exige a definição única **em código de nível superior**, com corpo
literal observado. Comentários, strings e expressões regulares não são
interpretados como declaração. O reconhecimento é conservador e **não
executa** código JS; variações não comprovadas continuam recusadas. Os
testes sintéticos negativos em `tests_scripts_inertes.rs` representam
essas hipóteses adversariais, **não** uma ocorrência comprovada em live.

A regressão anterior à correção falhou no CI
[RED 42290890168](https://github.com/franklinbaldo/ies/actions/runs/42290890168):
um teste novo falhou, 1.803 passaram e 23 permaneceram ignorados
preexistentes. O fluxo de envio criador e o orçamento de requisições
não foram ampliados; nenhum ensaio no SEI foi feito nesta fatia.
