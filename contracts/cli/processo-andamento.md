---
type: CommandContract
title: Registro de andamento no processo
operation: processo.andamento.registrar
command: ies processo andamento NUP --texto T [--confirmar]
effect: write-process-history
implementation: partial
---

# Registrar andamento no processo (#259)

O comando segue somente o link Web de Atualizar Andamento na barra do nó PROCESSO. A identidade do processo é opaca; URL assinada, cookie e segredo não aparecem na saída.

## Prévia e recusas

NUP e texto são validados antes de consultar a sessão. Texto vazio ou acima de 500 caracteres é recusado (saída 2, sem rede). A leitura do processo obtém o histórico anterior e a barra, que deve oferecer a ação procedimento_atualizar_andamento. A tela deve publicar exatamente um formulário frmAtividadeListar com controles hdnIdAtividade, hdnIdProtocolo igual ao processo, txaDescricao e sbmSalvar. Campos de usuário imprevistos são recusados. Sem --confirmar, deriva o formulário mas não faz POST.

## Escrita e verificação

Com --confirmar, envia um único POST no charset e action observados, sem seguir 302 e nunca repetir. Resposta esperada: 302 para procedimento_consultar_historico do mesmo processo. O 302 não prova escrita. Relê processo/histórico e exige visão igual, total_declarado acrescido de um, primeiro item com o texto normalizado por espaços e demais linhas antigas deslocadas em uma posição. A regra vale quando o texto já existia. Sem essa prova, saída 3 (efeito_incerto_no_andamento); não repetir sem conferência.

## Orçamento

Abertura do processo e histórico (variável conforme índice), mais 1 GET do formulário; com --confirmar soma 1 POST e nova leitura do processo/histórico. Sem consulta a marcadores ou propriedades. O comando informa as requisições realizadas na prévia e no sucesso.

## Evidência

[Cassette UnB](../../cassettes/unb/processo-andamento-registrar-unb--111df7e25e1e.md), [observação](../../docs/como-o-sei-funciona/registro-de-andamento.md), [decisão](../../docs/decisoes/processo-andamento.md). Faltam teste integrado de transporte/orçamento, replay e validação ao vivo; PR deve continuar Draft.
