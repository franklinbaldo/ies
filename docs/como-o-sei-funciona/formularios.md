---
type: SEIObservation
title: Formulários do SEI
description: Estado oculto, envio e validação
---

# Formulários

O link que abre um formulário e o seu atributo `action`
podem ter URLs assinadas **diferentes**. O `action` do POST
é publicado pelo servidor quando a tela de edição é aberta.

Campos ocultos podem incorporar identificadores, seleção de
listas, hash de tabela e estado que não deve ser inventado.
Além disso, o HTML não é necessariamente o payload efetivo:
o JavaScript da tela pode reescrever campos como
`hdnFlagDocumentoCadastro`, serializar seleções da lupa e
preencher identificadores de autocomplete.

Checkbox desmarcado pode não ser enviado; um `select`
pode ter valor vazio; botões `type=button` podem acionar
submissão programática; vários botões produzem efeitos diferentes.

O servidor pode recusar o gesto na própria página ou em
campos de validação, inclusive respondendo HTTP 200.
Uma tela de formulário desconhecida deve ser recusada sem agir.

**Regra empírica:** observar formulário fresco, reproduzir
o gesto efetivo conhecido e conferir o resultado do servidor.
GET/POST, isoladamente, não indicam leitura ou escrita.
