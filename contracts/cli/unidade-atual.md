---
type: CommandContract
title: Unidade exibida no Controle de Processos
operation: unidade.atual
command: ies unidade atual
effect: read
implementation: partial
source_commit: 3d4a38c3b5987e4a8fc7f97ded188c4876ed77cd
---

# Unidade exibida no Controle de Processos

`ies unidade atual` apresenta o rótulo da unidade ativa publicado em
`lnkInfraUnidade` no HTML da tela Controle de Processos do SEI Web.
O comando realiza somente **um GET autorizado** da mesma página que
`ies unidade listar-processos` usa, com a configuração local existente.

Não consulta outras unidades, não identifica a unidade por ID administrativo
e **não altera** a unidade ativa da sessão. Não percorre paginação para
identificar o rótulo. Com `--unidade`, exige que o texto exibido
coincida literalmente com o esperado; não normaliza a sigla.

A saída JSON contém `unidade_exibida` e
`origem: "controle_de_processos"`. Ausência do marcador no HTML
falha com `identificacao_da_unidade_ausente_no_controle`;
configuração de sessão ausente também falha explicitamente.

A evidência é derivada da primeira resposta da
[caixa da unidade](../evidencias/unidade-atual.md); não comprova
a lista integral de unidades oferecidas ao usuário nem a troca.
