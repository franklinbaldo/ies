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
