---
type: Contract
title: Prévia experimental de criação de marcador
description: Observação GET do formulário de criação e da cor publicada, sem submissão
---

# `ies marcador criar --nome N --cor C [--descricao D] [--confirmar]` (#880)

Fatia experimental da gestão de marcadores da unidade, não de sua aplicação
ao processo. Usa a sessão já existente sem relogin ou troca de unidade.

- `--confirmar` é recusado antes de abrir arquivos de acesso ou rede.
- Sem confirmação: nome não vazio (limite 50 unidades UTF-16), descrição
  opcional (limite 250, hipótese de comprimento conforme JS publicado),
  rótulo de cor literal oferecido pelo formulário.
- GET Controle → catálogo misto publicado → botão Novo publicado
  (`marcador_cadastrar`), com links relativos assinados da mesma instalação.
  Botões Novo idênticos repetidos são uma oferta; divergência é recusa.
  Não alterna/restaura filtro nem percorre páginas. Recusa nome ativo já
  observado na página; ausência não prova unicidade global.
- Confere unidade do formulário e controles publicados `txtNome`,
  `txaDescricao`, `hdnStaIcone`, `selStaIcone`, `sbmCadastrarMarcador`.
  A cor é selecionada pelo rótulo e conserva o código literal (inclusive 0).
  Dois rótulos distintos podem ter o mesmo código; não afirmar que são
  distintos após eventual gravação.
- JSON: unidade, nome, cor, codigo_cor, descricao, previa=true, criado=false,
  requisicoes=3. Texto humano indica que nada foi criado.
- Sem POST, payload, escrita, migration ou promessa de aceitação pelo SEI.

## Evidência

[HTML versionado Todos](https://github.com/franklinbaldo/todos/blob/caabe699bcf13b61b12dc5863638c8d90c0155c1/tests/paginas_reais/marcador_da_unidade_novo.html)
(blob 4128ecfec26072e904347bb7648f10c6e46ea87f) contém 32 opções não vazias
com 31 códigos distintos (Cinza/Cinza Escuro = 2; Preto = 0).
[Oráculo de comportamento](https://github.com/franklinbaldo/todos/blob/caabe699bcf13b61b12dc5863638c8d90c0155c1/src/todos/marcadores.py).
Os cards [UnB](../../cassettes/unb/marcador-ciclo-completo-unb--b24fe5a425ac.md)
e [GO](../../cassettes/go/marcador-ciclo-completo-go--ee16e8e455ef.md)
indexam as ações, não provam seus corpos. ZIP original/replay desta fatia
ainda não executados. Censo dos GETs originais escrito como teste ignorado
`cassette_previa_criacao_marcador_unb_go`, abrangido pelo replay oficial
`cassette_`. E2E loopback sintético escrito, pendente de CI.
