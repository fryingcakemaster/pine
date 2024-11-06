use std::mem::size_of;

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

declare_id!("59B4m6ZAUAda3pwFJN6Vy7Vf25eZpWtLCEViGXcQ6xXP");

#[program]
pub mod pine {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);

        let dex_state = &mut ctx.accounts.dex_state;
        dex_state.authority = ctx.accounts.authority.key();
        dex_state.order_count = 0;

        Ok(())
    }

    pub fn place_order(ctx: Context<PlaceOrder>, order_direction: OrderDirection, amount: u64, price: u64) -> Result<()> {
        msg!("Place Order...");

        let dex_state = &mut ctx.accounts.dex_state;
        msg!("dex_state: {:?}", dex_state.order_count);

        let order = Order {
            order_id: dex_state.order_count,
            owner: ctx.accounts.signer.key(),
            od: order_direction,
            amount,
            price,
            fulfilled: 0,
        };
        dex_state.order_count += 1;
        dex_state.orders.push(order);

        // Transfer tokens to DEX account
        msg!(">>> cpi accounts building...");

        let cpi_accounts = Transfer {
            from: ctx.accounts.user_token_account.to_account_info(),
            to: ctx.accounts.dex_token_account.to_account_info(),
            authority: ctx.accounts.signer.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        msg!(">>> is signer: {}", cpi_program.is_signer);
        
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        Ok(())
    }

    pub fn cancel_order(ctx: Context<CancelOrder>, order_id: u64) -> Result<()> {
        let dex_state = &mut ctx.accounts.dex_state;
        let order_index = dex_state
            .orders
            .iter()
            .position(|order| order.order_id == order_id && order.owner == ctx.accounts.user.key())
            .ok_or(ErrorCode::OrderNotFound)?;
        
        let order = dex_state.orders.remove(order_index);

        // Transfer tokens back to user
        let cpi_accounts = Transfer {
            from: ctx.accounts.dex_token_account.to_account_info(),
            to: ctx.accounts.user_token_account.to_account_info(),
            authority: ctx.accounts.dex_state.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, order.amount - order.fulfilled)?;

        Ok(())
    }

    pub fn match_orders(ctx: Context<MatchOrders>) -> Result<()> {
        let mut buy_orders: Vec<_> = ctx.accounts.dex_state.orders.iter().filter(|o| o.od == OrderDirection::Buy).map(|o| o.clone()).collect();
        let mut sell_orders: Vec<_> = ctx.accounts.dex_state.orders.iter().filter(|o| o.od == OrderDirection::Sell).map(|o| o.clone()).collect();

        buy_orders.sort_by(|a, b| b.price.cmp(&a.price));
        sell_orders.sort_by(|a, b| a.price.cmp(&b.price));

        for buy_order in buy_orders.iter_mut() {
            for sell_order in sell_orders.iter_mut() {
                if buy_order.price >= sell_order.price {
                    let match_amount = std::cmp::min(
                        buy_order.amount - buy_order.fulfilled,
                        sell_order.amount - sell_order.fulfilled,
                    );
                    
                    buy_order.fulfilled += match_amount;
                    sell_order.fulfilled += match_amount;

                    // Implement token transfers here
                }
            }
        }

        let dex_state = &mut ctx.accounts.dex_state;
        dex_state.orders.retain(|order| order.amount != order.fulfilled);

        Ok(())
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum OrderDirection {
    Buy,
    Sell,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Order {
    pub order_id: u64,
    pub owner: Pubkey,
    pub od: OrderDirection,
    pub amount: u64,
    pub price: u64,
    pub fulfilled: u64,
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    // the payer specified for an init constraint must be mutable.
    #[account(init, payer = authority, space = size_of::<DexState>() + 8)]
    pub dex_state: Account<'info, DexState>,

    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct DexState {
    pub authority: Pubkey,
    pub order_count: u64,
    pub orders: Vec<Order>,
}

#[derive(Accounts)]
pub struct PlaceOrder<'info> {
    pub dex_state: Account<'info, DexState>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub dex_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct CancelOrder<'info> {
    #[account(mut)]
    pub dex_state: Account<'info, DexState>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub dex_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Order not found")]
    OrderNotFound,
}

#[derive(Accounts)]
pub struct MatchOrders<'info> {
    #[account(mut)]
    pub dex_state: Account<'info, DexState>,
    #[account(mut)]
    pub authority: Signer<'info>,
}